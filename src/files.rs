// SPDX-License-Identifier: MIT OR Apache-2.0
//! The static-file handler. The security is in `resolve`: a request path becomes a file under the root, or nothing.

use std::io::SeekFrom;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use bytes::Bytes;
use futures_util::TryStreamExt;
use http::header::{
    ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, ETAG, IF_MODIFIED_SINCE,
    IF_NONE_MATCH, IF_RANGE, LAST_MODIFIED, LOCATION, RANGE, X_CONTENT_TYPE_OPTIONS,
};
use http::{HeaderValue, StatusCode};
use http_body_util::StreamBody;
use httpant::{Body, IntoResponse, PathParams, Request, Response};
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;

#[derive(Clone)]
pub(crate) struct Files {
    inner: Arc<Inner>,
}

struct Inner {
    root: PathBuf, // canonical
    dotfiles: bool,
}

/// Why a request path did not become a file. Both are answered `404`, so a probe learns nothing about what exists.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Refusal {
    /// Not a path we would ever serve (`..`, a drive or stream marker, a device name, a dotfile, a control character, a symlink out of the root).
    Forbidden,
    NotFound,
}

struct Info {
    method: http::Method,
    path: String,
    query: Option<String>,
    headers: http::HeaderMap,
}

pub(crate) struct Target {
    pub(crate) path: PathBuf,
    pub(crate) meta: std::fs::Metadata,
}

impl Files {
    pub(crate) fn new(root: &Path, dotfiles: bool) -> std::io::Result<Self> {
        let root = root.canonicalize()?;
        if !root.is_dir() {
            return Err(std::io::Error::other("not a directory"));
        }
        Ok(Self {
            inner: Arc::new(Inner { root, dotfiles }),
        })
    }

    pub(crate) async fn handle(&self, req: Request) -> Response {
        let rest = req
            .extensions()
            .get::<PathParams>()
            .and_then(|p| p.get("rest"))
            .unwrap_or("")
            .to_owned();
        // The request body is not `Sync`, so only what the answer needs is kept across the awaits below.
        let info = Info {
            method: req.method().clone(),
            path: req.uri().path().to_owned(),
            query: req.uri().query().map(str::to_owned),
            headers: req.headers().clone(),
        };
        drop(req);
        let mut res = self.respond(&info, &rest).await;
        res.headers_mut()
            .insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
        res
    }

    async fn respond(&self, req: &Info, rest: &str) -> Response {
        let Ok(target) = self.resolve(rest).await else {
            return not_found();
        };
        let mut path = target.path;
        let mut meta = target.meta;
        if meta.is_dir() {
            // `/docs` must become `/docs/` so relative links in its index page work.
            if !req.path.ends_with('/') {
                let mut to = format!("{}/", req.path);
                if let Some(q) = &req.query {
                    to.push('?');
                    to.push_str(q);
                }
                let mut res = StatusCode::PERMANENT_REDIRECT.into_response();
                if let Ok(v) = HeaderValue::from_str(&to) {
                    res.headers_mut().insert(LOCATION, v);
                }
                return res;
            }
            // `index.html` may itself be a symlink out of the root, so it goes through the same check.
            let Ok(index) = self.resolve_under(&path, "index.html").await else {
                return not_found();
            };
            if !index.meta.is_file() {
                return not_found();
            }
            path = index.path;
            meta = index.meta;
        }
        serve_file(req, &path, &meta).await
    }

    /// Turn the decoded `*rest` capture into an existing path under the root.
    pub(crate) async fn resolve(&self, rest: &str) -> Result<Target, Refusal> {
        let mut path = self.inner.root.clone();
        for part in rest.split('/') {
            if part.is_empty() {
                continue; // a trailing slash, or `//`
            }
            if !self.component_ok(part) {
                return Err(Refusal::Forbidden);
            }
            path.push(part);
        }
        self.confirm(path).await
    }

    async fn resolve_under(&self, dir: &Path, name: &str) -> Result<Target, Refusal> {
        self.confirm(dir.join(name)).await
    }

    /// Symlinks and junctions: the real location must still be under the real root.
    async fn confirm(&self, path: PathBuf) -> Result<Target, Refusal> {
        let real = tokio::fs::canonicalize(&path)
            .await
            .map_err(|_| Refusal::NotFound)?;
        if !real.starts_with(&self.inner.root) {
            return Err(Refusal::Forbidden);
        }
        let meta = tokio::fs::metadata(&real)
            .await
            .map_err(|_| Refusal::NotFound)?;
        Ok(Target { path: real, meta })
    }

    fn component_ok(&self, part: &str) -> bool {
        if part == "." || part == ".." {
            return false;
        }
        if part.starts_with('.') && !self.inner.dotfiles {
            return false;
        }
        // `\` is a separator on Windows, `:` starts a drive or an alternate data stream (`file.txt::$DATA`), controls include NUL, and the rest
        // are characters Windows refuses or treats as wildcards.
        if part
            .chars()
            .any(|c| c.is_control() || "\\:*?\"<>|".contains(c))
        {
            return false;
        }
        // Windows ignores trailing dots and spaces, so `index.html.` can name something other than it looks like.
        if part.ends_with('.') || part.ends_with(' ') {
            return false;
        }
        !is_windows_device(part)
    }
}

/// `CON`, `PRN`, `AUX`, `NUL`, `COM1`..`COM9`, `LPT1`..`LPT9`, with or without an extension, in any case. Refused on every platform so a site
/// behaves the same wherever it is served.
fn is_windows_device(part: &str) -> bool {
    let stem = part
        .split('.')
        .next()
        .unwrap_or(part)
        .trim_end()
        .to_ascii_uppercase();
    matches!(
        stem.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
    ) || ((stem.starts_with("COM") || stem.starts_with("LPT"))
        && stem.len() == 4
        && matches!(stem.as_bytes()[3], b'1'..=b'9'))
}

fn not_found() -> Response {
    (StatusCode::NOT_FOUND, "not found").into_response()
}

fn etag(meta: &std::fs::Metadata) -> String {
    let nanos = meta
        .modified()
        .ok()
        .and_then(|m| m.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_nanos());
    format!("W/\"{:x}-{:x}\"", meta.len(), nanos)
}

fn truncate_secs(t: SystemTime) -> SystemTime {
    match t.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(d.as_secs()),
        Err(_) => t,
    }
}

fn plain(status: StatusCode) -> Response {
    status.into_response()
}

async fn serve_file(req: &Info, path: &Path, meta: &std::fs::Metadata) -> Response {
    let len = meta.len();
    let etag = etag(meta);
    let modified = meta.modified().ok();
    let head = req.method == http::Method::HEAD;

    // Conditional request: `If-None-Match` wins over `If-Modified-Since` (RFC 9110 section 13.1.3).
    let not_modified = match req.headers.get(IF_NONE_MATCH).and_then(|v| v.to_str().ok()) {
        Some(inm) => inm.split(',').any(|t| t.trim() == "*" || t.trim() == etag),
        None => match (
            req.headers
                .get(IF_MODIFIED_SINCE)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| httpdate::parse_http_date(v).ok()),
            modified,
        ) {
            (Some(since), Some(m)) => truncate_secs(m) <= since,
            _ => false,
        },
    };

    let mut builder = http::Response::builder()
        .header(ETAG, &etag)
        .header(ACCEPT_RANGES, "bytes");
    if let Some(m) = modified {
        builder = builder.header(LAST_MODIFIED, httpdate::fmt_http_date(m));
    }
    if not_modified {
        return builder
            .status(StatusCode::NOT_MODIFIED)
            .body(Body::empty())
            .unwrap_or_else(|_| plain(StatusCode::INTERNAL_SERVER_ERROR));
    }
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    let ctype = if mime.type_() == mime_guess::mime::TEXT {
        format!("{mime}; charset=utf-8")
    } else {
        mime.to_string()
    };
    builder = builder.header(CONTENT_TYPE, ctype);

    let (status, start, count) = match parse_range(req, len, &etag) {
        RangeOutcome::Whole => (StatusCode::OK, 0, len),
        RangeOutcome::Part(s, e) => {
            builder = builder.header(CONTENT_RANGE, format!("bytes {s}-{e}/{len}"));
            (StatusCode::PARTIAL_CONTENT, s, e - s + 1)
        }
        RangeOutcome::Unsatisfiable => {
            return http::Response::builder()
                .status(StatusCode::RANGE_NOT_SATISFIABLE)
                .header(CONTENT_RANGE, format!("bytes */{len}"))
                .body(Body::empty())
                .unwrap_or_else(|_| plain(StatusCode::INTERNAL_SERVER_ERROR));
        }
    };
    builder = builder.header(CONTENT_LENGTH, count).status(status);
    let body = if head {
        Body::empty()
    } else {
        let Ok(mut file) = tokio::fs::File::open(path).await else {
            return not_found();
        };
        if start > 0 && file.seek(SeekFrom::Start(start)).await.is_err() {
            return plain(StatusCode::INTERNAL_SERVER_ERROR);
        }
        let stream =
            ReaderStream::new(file.take(count)).map_ok(|b: Bytes| http_body::Frame::data(b));
        Body::new(StreamBody::new(stream))
    };
    builder
        .body(body)
        .unwrap_or_else(|_| plain(StatusCode::INTERNAL_SERVER_ERROR))
}

enum RangeOutcome {
    Whole,
    Part(u64, u64),
    Unsatisfiable,
}

/// One `bytes=a-b`, `bytes=a-` or `bytes=-n` range. Several ranges, another unit, a malformed header, or an `If-Range` that does not match
/// all give the whole file (RFC 9110 allows a server to ignore `Range`).
fn parse_range(req: &Info, len: u64, etag: &str) -> RangeOutcome {
    let Some(h) = req.headers.get(RANGE).and_then(|v| v.to_str().ok()) else {
        return RangeOutcome::Whole;
    };
    if let Some(ir) = req.headers.get(IF_RANGE).and_then(|v| v.to_str().ok())
        && ir != etag
    {
        return RangeOutcome::Whole;
    }
    let Some(spec) = h.strip_prefix("bytes=") else {
        return RangeOutcome::Whole;
    };
    let Some((a, b)) = spec.split_once('-') else {
        return RangeOutcome::Whole;
    };
    if spec.contains(',') {
        return RangeOutcome::Whole;
    }
    let (a, b) = (a.trim(), b.trim());
    match (a.parse::<u64>(), b.parse::<u64>()) {
        (Ok(s), Ok(e)) if s <= e => {
            if s >= len {
                RangeOutcome::Unsatisfiable
            } else {
                RangeOutcome::Part(s, e.min(len - 1))
            }
        }
        (Ok(s), Err(_)) if b.is_empty() => {
            if s >= len {
                RangeOutcome::Unsatisfiable
            } else {
                RangeOutcome::Part(s, len - 1)
            }
        }
        (Err(_), Ok(n)) if a.is_empty() && n > 0 => {
            if len == 0 {
                RangeOutcome::Unsatisfiable
            } else {
                RangeOutcome::Part(len.saturating_sub(n), len - 1)
            }
        }
        _ => RangeOutcome::Whole,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_device_names_are_recognised_in_any_case_and_with_extensions() {
        for yes in [
            "CON",
            "con",
            "Nul.txt",
            "aux.tar.gz",
            "PRN",
            "com1",
            "COM9.log",
            "LPT1",
            "lpt9.x",
            "CONIN$",
            "con .txt",
        ] {
            assert!(is_windows_device(yes), "{yes}");
        }
        // The controls: names that merely look similar.
        for no in [
            "console",
            "com0",
            "com10",
            "lpt",
            "comma.txt",
            "index.html",
            "nullable",
            "a.con",
        ] {
            assert!(!is_windows_device(no), "{no}");
        }
    }

    #[test]
    fn component_rules() {
        let f = Files::new(Path::new("."), false).unwrap();
        for bad in [
            "..",
            ".",
            ".env",
            "a:b",
            r"a\b",
            "a\0b",
            "x.",
            "x ",
            "a*b",
            "a?b",
            "CON",
            "file.txt::$DATA",
        ] {
            assert!(!f.component_ok(bad), "{bad:?}");
        }
        for good in [
            "index.html",
            "a-b_c.txt",
            "caf\u{e9}.html",
            "a b.txt",
            "file.tar.gz",
        ] {
            assert!(f.component_ok(good), "{good:?}");
        }
        assert!(
            Files::new(Path::new("."), true)
                .unwrap()
                .component_ok(".well-known")
        );
    }
}
