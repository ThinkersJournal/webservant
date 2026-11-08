// SPDX-License-Identifier: MIT OR Apache-2.0
//! The reverse proxy: requests under a path prefix are forwarded to one upstream and its answer is streamed back.
//!
//! What is deliberately decided here: the client's `X-Forwarded-*` and `Forwarded` headers are never believed (we write our own from what
//! HTTPAnt established), hop-by-hop headers stop at the proxy, a path that could climb out of the prefix with dot segments is refused, an
//! upgrade (WebSocket) is refused out loud, and an upstream failure is a `502`/`504` that says nothing about the upstream.

use std::sync::Arc;
use std::time::Duration;

use http::header::{self, HeaderMap, HeaderName, HeaderValue};
use http::{Method, StatusCode};
use httpant::server::{Origin, Scheme};
use httpant::{Client, IntoResponse, Request, Response};

use crate::{Error, Result};

/// Hop-by-hop headers (RFC 9110 section 7.6.1): meaningful for one connection only, never forwarded. Names listed in `Connection` are too.
const HOP_BY_HOP: [&str; 9] = [
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "proxy-connection",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
];

/// What the client said about itself and the proxy must not believe.
const FORWARDING: [&str; 4] = [
    "forwarded",
    "x-forwarded-for",
    "x-forwarded-proto",
    "x-forwarded-host",
];

#[derive(Clone)]
pub(crate) struct Proxy {
    inner: Arc<Inner>,
}

struct Inner {
    prefix: String,
    /// `scheme://authority` plus the base path with no trailing slash.
    base: String,
    client: Client,
}

impl Proxy {
    pub(crate) fn new(prefix: &str, upstream: &str) -> Result<Self> {
        let bad = |reason: &str| Error::Proxy {
            what: format!("{prefix} -> {}", redact(upstream)),
            reason: reason.to_owned(),
        };
        if !prefix.starts_with('/')
            || prefix.len() < 2
            || prefix.ends_with('/')
            || prefix.contains(['?', '#', '*', ':'])
        {
            return Err(bad(
                "the prefix must start with `/`, name a path such as `/api`, and not end with `/`",
            ));
        }
        let uri: http::Uri = upstream
            .parse()
            .map_err(|_| bad("the upstream is not a URL"))?;
        let scheme = uri.scheme_str().unwrap_or("");
        if scheme != "http" && scheme != "https" {
            return Err(bad("the upstream must start with `http://` or `https://`"));
        }
        let authority = uri
            .authority()
            .ok_or_else(|| bad("the upstream has no host"))?;
        if authority.as_str().contains('@') {
            return Err(bad("the upstream must not contain a user name or password"));
        }
        if uri.query().is_some() || upstream.contains('#') {
            return Err(bad("the upstream must not contain a query or fragment"));
        }
        let base = format!("{scheme}://{authority}{}", uri.path().trim_end_matches('/'));
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(60))
            .read_timeout(Duration::from_secs(60))
            .build()
            .map_err(Error::from)?;
        Ok(Self {
            inner: Arc::new(Inner {
                prefix: prefix.to_owned(),
                base,
                client,
            }),
        })
    }

    pub(crate) fn prefix(&self) -> &str {
        &self.inner.prefix
    }

    pub(crate) async fn handle(&self, req: Request) -> Response {
        let (parts, body) = req.into_parts();
        let path = parts.uri.path();
        let rest = path.strip_prefix(&self.inner.prefix).unwrap_or("");
        // `/apix` is not under `/api`; the router sends only `/api` and `/api/...`, but this is the check that matters.
        if !(rest.is_empty() || rest.starts_with('/')) {
            return StatusCode::NOT_FOUND.into_response();
        }
        if has_dot_segment(rest) {
            return (StatusCode::BAD_REQUEST, "bad request path").into_response();
        }
        if wants_upgrade(&parts.headers) {
            return (
                StatusCode::NOT_IMPLEMENTED,
                "this server does not forward upgrades (WebSocket) yet",
            )
                .into_response();
        }
        let mut url = format!(
            "{}{}",
            self.inner.base,
            if rest.is_empty() { "/" } else { rest }
        );
        if let Some(q) = parts.uri.query() {
            url.push('?');
            url.push_str(q);
        }

        let mut up = self.inner.client.request(parts.method.clone(), &url);
        for (name, value) in forwardable(&parts.headers) {
            up = up.header(name.as_str(), value.as_bytes());
        }
        if let Some(host) = parts.headers.get(header::HOST) {
            up = up.header("x-forwarded-host", host.as_bytes());
        }
        if let Some(o) = parts.extensions.get::<Origin>() {
            up = up.header("x-forwarded-for", o.client.to_string());
            let proto = if o.scheme == Scheme::Https {
                "https"
            } else {
                "http"
            };
            up = up.header("x-forwarded-proto", proto);
        }
        let has_body = parts.method != Method::GET && parts.method != Method::HEAD;
        if has_body {
            up = up.body_stream(body);
        }
        match up.send().await {
            Ok(answer) => {
                let status = answer.status();
                let headers = answer.headers().clone();
                let mut res = Response::new(answer.into_body());
                *res.status_mut() = status;
                for (name, value) in forwardable(&headers) {
                    res.headers_mut().append(name.clone(), value.clone());
                }
                res
            }
            Err(e) => {
                use httpant::Error as E;
                match e {
                    E::Timeout(_) | E::ConnectTimeout(_) | E::ReadTimeout(_) => {
                        (StatusCode::GATEWAY_TIMEOUT, "gateway timeout").into_response()
                    }
                    _ => (StatusCode::BAD_GATEWAY, "bad gateway").into_response(),
                }
            }
        }
    }
}

fn redact(upstream: &str) -> String {
    // No user name, query or fragment in anything we print.
    let end = upstream.find(['?', '#']).unwrap_or(upstream.len());
    let s = &upstream[..end];
    match s.split_once("://") {
        Some((scheme, rest)) => format!(
            "{scheme}://{}",
            rest.rsplit_once('@').map_or(rest, |(_, h)| h)
        ),
        None => s.to_owned(),
    }
}

/// The headers worth forwarding: not hop-by-hop, not named by `Connection`, not `Host`, and not what the client claimed about forwarding.
fn forwardable(headers: &HeaderMap) -> impl Iterator<Item = (&HeaderName, &HeaderValue)> {
    let named_by_connection: Vec<String> = headers
        .get_all(header::CONNECTION)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(','))
        .map(|t| t.trim().to_ascii_lowercase())
        .filter(|t| !t.is_empty())
        .collect();
    headers.iter().filter(move |(n, _)| {
        let n = n.as_str();
        n != "host"
            && n != "expect"
            && !HOP_BY_HOP.contains(&n)
            && !FORWARDING.contains(&n)
            && !named_by_connection.iter().any(|c| c == n)
    })
}

fn wants_upgrade(headers: &HeaderMap) -> bool {
    headers.contains_key(header::UPGRADE)
        || headers
            .get_all(header::CONNECTION)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .flat_map(|v| v.split(','))
            .any(|t| t.trim().eq_ignore_ascii_case("upgrade"))
}

/// A `.` or `..` segment, raw or percent-encoded: forwarded, it would let a request climb out of the prefix at an upstream that normalises.
fn has_dot_segment(path: &str) -> bool {
    path.split('/').any(|seg| {
        let decoded = decode(seg);
        decoded == "."
            || decoded == ".."
            || decoded.contains('\\') && decoded.split('\\').any(|s| s == ".." || s == ".")
    })
}

fn decode(seg: &str) -> String {
    let b = seg.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%'
            && i + 2 < b.len()
            && let (Some(h), Some(l)) = (
                (b[i + 1] as char).to_digit(16),
                (b[i + 2] as char).to_digit(16),
            )
        {
            out.push((h * 16 + l) as u8);
            i += 3;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dot_segments_are_found_raw_and_encoded() {
        for bad in [
            "/..",
            "/a/../b",
            "/%2e%2e/x",
            "/%2E/x",
            "/a/%2e%2E",
            "/.",
            "/..%5c",
            "/a%5c..%5cb",
        ] {
            assert!(has_dot_segment(bad), "{bad}");
        }
        for ok in [
            "",
            "/",
            "/a/b.c",
            "/.well-known/x",
            "/a..b",
            "/...",
            "/%2e%2ex",
            "/a%2fb",
        ] {
            assert!(!has_dot_segment(ok), "{ok}");
        }
    }

    #[test]
    fn upstream_and_prefix_are_validated() {
        for (p, u) in [
            ("api", "http://127.0.0.1:1"),
            ("/", "http://127.0.0.1:1"),
            ("/api/", "http://127.0.0.1:1"),
            ("/api", "ftp://x"),
            ("/api", "http://u:p@host"),
            ("/api", "http://host/?q=1"),
            ("/api", "not a url"),
            ("/a*", "http://host"),
        ] {
            assert!(Proxy::new(p, u).is_err(), "{p} {u}");
        }
        assert!(Proxy::new("/api", "http://127.0.0.1:3000").is_ok());
        assert!(Proxy::new("/api/v1", "https://example.org/base/").is_ok());
    }

    #[test]
    fn the_error_text_never_holds_a_password_or_query() {
        let e = Proxy::new("/api", "ftp://user:secretpw@host/?token=abc")
            .err()
            .unwrap()
            .to_string();
        assert!(!e.contains("secretpw") && !e.contains("token=abc"), "{e}");
    }
}
