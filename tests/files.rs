// SPDX-License-Identifier: MIT OR Apache-2.0
//! The static-file handler over real sockets. Requests are written by hand so that nothing normalises a path before the server sees it.

use std::fs;
use std::path::Path;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use webservant::{RunningSite, Site};

struct Reply {
    status: u16,
    head: String,
    body: Vec<u8>,
}

impl Reply {
    fn header(&self, name: &str) -> Option<String> {
        self.head.lines().skip(1).find_map(|l| {
            let (k, v) = l.split_once(':')?;
            k.eq_ignore_ascii_case(name).then(|| v.trim().to_owned())
        })
    }
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
}

async fn raw(site: &RunningSite, method: &str, target: &str, extra: &[(&str, &str)]) -> Reply {
    let mut s = TcpStream::connect(site.addr()).await.unwrap();
    let mut req = format!("{method} {target} HTTP/1.1\r\nHost: test\r\nConnection: close\r\n");
    for (k, v) in extra {
        req.push_str(&format!("{k}: {v}\r\n"));
    }
    req.push_str("\r\n");
    s.write_all(req.as_bytes()).await.unwrap();
    let mut buf = Vec::new();
    s.read_to_end(&mut buf).await.unwrap();
    let split = buf
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("a complete response head");
    let head = String::from_utf8_lossy(&buf[..split]).into_owned();
    let status = head.split(' ').nth(1).unwrap().parse().unwrap();
    // `Connection: close` and a Content-Length (or chunked) body: chunked is decoded only for the shapes these tests produce.
    let mut body = buf[split + 4..].to_vec();
    if head
        .to_ascii_lowercase()
        .contains("transfer-encoding: chunked")
    {
        body = dechunk(&body);
    }
    Reply { status, head, body }
}

fn dechunk(mut b: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let eol = b.windows(2).position(|w| w == b"\r\n").unwrap();
        let n = usize::from_str_radix(std::str::from_utf8(&b[..eol]).unwrap().trim(), 16).unwrap();
        if n == 0 {
            return out;
        }
        out.extend_from_slice(&b[eol + 2..eol + 2 + n]);
        b = &b[eol + 2 + n + 2..];
    }
}

/// A site root with: index.html, a.txt (0123456789), sub/index.html, .secret, and a `secret.txt` OUTSIDE the root.
async fn fixture() -> (tempfile::TempDir, RunningSite) {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("public");
    fs::create_dir_all(root.join("sub")).unwrap();
    fs::write(root.join("index.html"), "<h1>home</h1>").unwrap();
    fs::write(root.join("a.txt"), "0123456789").unwrap();
    fs::write(root.join("sub").join("index.html"), "sub page").unwrap();
    fs::write(root.join(".secret"), "dot").unwrap();
    fs::write(tmp.path().join("secret.txt"), "OUTSIDE").unwrap();
    let site = Site::new("test")
        .files(&root)
        .bind("127.0.0.1:0")
        .start()
        .await
        .unwrap();
    (tmp, site)
}

#[tokio::test]
async fn serves_a_file_an_index_and_a_subdirectory() {
    let (_t, site) = fixture().await;
    let r = raw(&site, "GET", "/a.txt", &[]).await;
    assert_eq!((r.status, r.text()), (200, "0123456789".into()));
    assert_eq!(
        r.header("content-type").unwrap(),
        "text/plain; charset=utf-8"
    );
    assert_eq!(r.header("x-content-type-options").unwrap(), "nosniff");
    assert_eq!(raw(&site, "GET", "/", &[]).await.text(), "<h1>home</h1>");
    assert_eq!(raw(&site, "GET", "/sub/", &[]).await.text(), "sub page");
    let r = raw(&site, "GET", "/sub", &[]).await;
    assert_eq!(r.status, 308);
    assert_eq!(r.header("location").unwrap(), "/sub/");
}

#[tokio::test]
async fn head_has_the_length_and_no_body() {
    let (_t, site) = fixture().await;
    let r = raw(&site, "HEAD", "/a.txt", &[]).await;
    assert_eq!(r.status, 200);
    assert_eq!(r.header("content-length").unwrap(), "10");
    assert!(r.body.is_empty());
}

#[tokio::test]
async fn nothing_outside_the_root_is_reachable() {
    let (_t, site) = fixture().await;
    // The same file is reachable by none of these, and the answer is `404` (not `403`), identical to a missing file.
    let missing = raw(&site, "GET", "/nope.txt", &[]).await;
    assert_eq!(missing.status, 404);
    for target in [
        "/../secret.txt",
        "/%2e%2e/secret.txt",
        "/%2E%2E/secret.txt",
        "/..%2fsecret.txt",
        "/%2e%2e%2fsecret.txt",
        "/sub/../../secret.txt",
        "/sub/%2e%2e/%2e%2e/secret.txt",
        "/..%5csecret.txt",
        "/sub\\..\\..\\secret.txt",
        "/%2e%2e%5csecret.txt",
        "//secret.txt",
        "/C:/Windows/win.ini",
        "/a.txt::$DATA",
        "/a.txt:stream",
        "/a.txt.",
        "/a.txt%20",
        "/CON",
        "/nul.txt",
        "/com1",
        "/LPT9.html",
        "/.secret",
        "/sub/%2e",
        "/a.txt%00.png",
    ] {
        let r = raw(&site, "GET", target, &[]).await;
        assert!(
            !r.text().contains("OUTSIDE") && !r.text().contains("dot"),
            "{target} leaked a file"
        );
        assert!(
            r.status == 404 || r.status == 400,
            "{target} answered {}",
            r.status
        );
        if r.status == 404 {
            assert_eq!(
                r.text(),
                missing.text(),
                "{target} differs from a missing file"
            );
        }
    }
}

#[cfg(unix)]
fn symlink(from: &Path, to: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(from, to)
}
#[cfg(windows)]
fn symlink(from: &Path, to: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(from, to)
}

#[tokio::test]
async fn a_symlink_out_of_the_root_is_not_followed() {
    let (t, site) = fixture().await;
    let link = t.path().join("public").join("escape.txt");
    if let Err(e) = symlink(&t.path().join("secret.txt"), &link) {
        // Creating a symlink on Windows needs a privilege CI runners and developer mode have, and an ordinary account may not.
        eprintln!("SKIPPED: cannot create a symlink here ({e}); this case did not run");
        return;
    }
    let r = raw(&site, "GET", "/escape.txt", &[]).await;
    assert_eq!(r.status, 404);
    assert!(!r.text().contains("OUTSIDE"));
    // The control: a symlink that stays inside the root is served.
    let inside = t.path().join("public").join("inside.txt");
    symlink(&t.path().join("public").join("a.txt"), &inside).unwrap();
    assert_eq!(
        raw(&site, "GET", "/inside.txt", &[]).await.text(),
        "0123456789"
    );
}

#[tokio::test]
async fn dotfiles_are_served_only_when_asked() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join(".well-known"), "wk").unwrap();
    let site = Site::new("t")
        .files(tmp.path())
        .bind("127.0.0.1:0")
        .dotfiles(true)
        .start()
        .await
        .unwrap();
    assert_eq!(raw(&site, "GET", "/.well-known", &[]).await.text(), "wk");
}

#[tokio::test]
async fn conditional_requests_answer_304() {
    let (_t, site) = fixture().await;
    let first = raw(&site, "GET", "/a.txt", &[]).await;
    let etag = first.header("etag").unwrap();
    let lm = first.header("last-modified").unwrap();
    let r = raw(&site, "GET", "/a.txt", &[("If-None-Match", &etag)]).await;
    assert_eq!((r.status, r.body.len()), (304, 0));
    let r = raw(&site, "GET", "/a.txt", &[("If-Modified-Since", &lm)]).await;
    assert_eq!(r.status, 304);
    // The control: a different tag gets the file.
    let r = raw(&site, "GET", "/a.txt", &[("If-None-Match", "W/\"other\"")]).await;
    assert_eq!(r.status, 200);
    // A tag that matches wins over a date that would say "modified".
    let r = raw(
        &site,
        "GET",
        "/a.txt",
        &[
            ("If-None-Match", &etag),
            ("If-Modified-Since", "Thu, 01 Jan 1970 00:00:00 GMT"),
        ],
    )
    .await;
    assert_eq!(r.status, 304);
}

#[tokio::test]
async fn ranges() {
    let (_t, site) = fixture().await;
    let r = raw(&site, "GET", "/a.txt", &[("Range", "bytes=2-4")]).await;
    assert_eq!((r.status, r.text()), (206, "234".into()));
    assert_eq!(r.header("content-range").unwrap(), "bytes 2-4/10");
    assert_eq!(
        raw(&site, "GET", "/a.txt", &[("Range", "bytes=7-")])
            .await
            .text(),
        "789"
    );
    assert_eq!(
        raw(&site, "GET", "/a.txt", &[("Range", "bytes=-3")])
            .await
            .text(),
        "789"
    );
    // An end past the file is clamped.
    assert_eq!(
        raw(&site, "GET", "/a.txt", &[("Range", "bytes=8-99")])
            .await
            .text(),
        "89"
    );
    let r = raw(&site, "GET", "/a.txt", &[("Range", "bytes=10-12")]).await;
    assert_eq!(r.status, 416);
    assert_eq!(r.header("content-range").unwrap(), "bytes */10");
    // Not understood: the whole file, not an error.
    for odd in ["bytes=0-1,4-5", "items=1-2", "bytes=x-y", "bytes=5-2"] {
        let r = raw(&site, "GET", "/a.txt", &[("Range", odd)]).await;
        assert_eq!((r.status, r.text()), (200, "0123456789".into()), "{odd}");
    }
}

#[tokio::test]
async fn bad_settings_are_named_before_anything_listens() {
    let e = Site::new("t")
        .bind("127.0.0.1:0")
        .start()
        .await
        .unwrap_err();
    assert!(e.to_string().contains(".files("), "{e}");
    let e = Site::new("t")
        .files("definitely-not-a-directory-here")
        .bind("127.0.0.1:0")
        .start()
        .await
        .unwrap_err();
    assert!(
        e.to_string().contains("definitely-not-a-directory-here"),
        "{e}"
    );
    let tmp = tempfile::tempdir().unwrap();
    let e = Site::new("t")
        .files(tmp.path())
        .tls_pem_files(tmp.path().join("no.pem"), tmp.path().join("no.key"))
        .bind("127.0.0.1:0")
        .start()
        .await
        .unwrap_err();
    assert!(e.to_string().contains("no.pem"), "{e}");
}
