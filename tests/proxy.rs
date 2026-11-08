// SPDX-License-Identifier: MIT OR Apache-2.0
//! The reverse proxy against a real upstream (an HTTPAnt server that reports what it received), over real sockets. Requests are written by
//! hand so that the exact bytes the proxy sees are known.

use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use http::Method;
use httpant::{Request, Response, Server};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use webservant::{RunningSite, Site};

struct Reply {
    status: u16,
    head: String,
    body: Vec<u8>,
}

impl Reply {
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
    fn header(&self, name: &str) -> Option<String> {
        self.head.lines().skip(1).find_map(|l| {
            let (k, v) = l.split_once(':')?;
            k.eq_ignore_ascii_case(name).then(|| v.trim().to_owned())
        })
    }
}

async fn raw(
    addr: std::net::SocketAddr,
    method: &str,
    target: &str,
    headers: &[(&str, &str)],
    body: &[u8],
) -> Reply {
    let mut s = TcpStream::connect(addr).await.unwrap();
    let mut req =
        format!("{method} {target} HTTP/1.1\r\nHost: front.test\r\nConnection: close\r\n");
    for (k, v) in headers {
        req.push_str(&format!("{k}: {v}\r\n"));
    }
    if !body.is_empty() {
        req.push_str(&format!("Content-Length: {}\r\n", body.len()));
    }
    req.push_str("\r\n");
    s.write_all(req.as_bytes()).await.unwrap();
    s.write_all(body).await.unwrap();
    let mut buf = Vec::new();
    s.read_to_end(&mut buf).await.unwrap();
    let split = buf
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("a complete response head");
    let head = String::from_utf8_lossy(&buf[..split]).into_owned();
    let status = head.split(' ').nth(1).unwrap().parse().unwrap();
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

/// An upstream that answers every request with a report: `METHOD target`, then `name: value` lines, a blank line, the body it received.
/// A request to `/status/<n>` answers with status `n` and two custom headers. `calls` counts requests.
async fn upstream() -> (httpant::RunningServer, Arc<AtomicUsize>) {
    let calls = Arc::new(AtomicUsize::new(0));
    let c = Arc::clone(&calls);
    let handler = move |req: Request| {
        let c = Arc::clone(&c);
        async move {
            c.fetch_add(1, Ordering::SeqCst);
            let (parts, body) = req.into_parts();
            let target = parts
                .uri
                .path_and_query()
                .map(|p| p.as_str().to_owned())
                .unwrap_or_default();
            if let Some(n) = parts.uri.path().strip_prefix("/status/") {
                let mut res = Response::new(httpant::Body::from_bytes("teapot body"));
                *res.status_mut() = http::StatusCode::from_u16(n.parse().unwrap()).unwrap();
                res.headers_mut()
                    .insert("x-upstream", "yes".parse().unwrap());
                res.headers_mut()
                    .insert("keep-alive", "timeout=5".parse().unwrap());
                return res;
            }
            let mut report = format!("{} {target}\n", parts.method);
            for (k, v) in &parts.headers {
                report.push_str(&format!("{}: {}\n", k.as_str(), v.to_str().unwrap_or("?")));
            }
            let body = body.bytes(1 << 20).await.unwrap();
            report.push('\n');
            let mut out = report.into_bytes();
            out.extend_from_slice(&body);
            Response::new(httpant::Body::from_bytes(out))
        }
    };
    let mut s = Server::new();
    for m in [
        Method::GET,
        Method::HEAD,
        Method::POST,
        Method::PUT,
        Method::PATCH,
        Method::DELETE,
        Method::OPTIONS,
    ] {
        s = s
            .route(m.clone(), "/", handler.clone())
            .route(m, "/*rest", handler.clone());
    }
    (s.bind("127.0.0.1:0").await.unwrap(), calls)
}

fn seen_header(report: &str, name: &str) -> Option<String> {
    report
        .split("\n\n")
        .next()?
        .lines()
        .skip(1)
        .filter_map(|l| l.split_once(": "))
        .find(|(k, _)| *k == name)
        .map(|(_, v)| v.to_owned())
}

async fn front(up: &httpant::RunningServer, files: bool) -> (tempfile::TempDir, RunningSite) {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("public");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("a.txt"), "file a").unwrap();
    let mut site = Site::new("front.test")
        .bind("127.0.0.1:0")
        .proxy("/api", format!("http://{}", up.addr()));
    if files {
        site = site.files(&root);
    }
    (tmp, site.start().await.unwrap())
}

#[tokio::test]
async fn forwards_path_with_prefix_removed_query_method_and_body() {
    let (up, _) = upstream().await;
    let (_t, site) = front(&up, true).await;
    let r = raw(site.addr(), "GET", "/api/users/7?x=1&y=%20z", &[], b"").await;
    assert_eq!(r.status, 200);
    assert!(
        r.text().starts_with("GET /users/7?x=1&y=%20z\n"),
        "{}",
        r.text()
    );
    // The prefix alone is the upstream's root.
    assert!(
        raw(site.addr(), "GET", "/api", &[], b"")
            .await
            .text()
            .starts_with("GET /\n")
    );
    assert!(
        raw(site.addr(), "GET", "/api/", &[], b"")
            .await
            .text()
            .starts_with("GET /\n")
    );
    // A body goes through whole, with its method.
    let payload: Vec<u8> = (0..100_000u32).map(|i| (i % 251) as u8).collect();
    let r = raw(
        site.addr(),
        "POST",
        "/api/echo",
        &[("Content-Type", "application/octet-stream")],
        &payload,
    )
    .await;
    assert!(r.text().starts_with("POST /echo\n"));
    let at = r.body.windows(2).position(|w| w == b"\n\n").unwrap() + 2;
    assert_eq!(
        &r.body[at..],
        &payload[..],
        "the body was altered on the way"
    );
    assert_eq!(
        seen_header(&r.text(), "content-type").unwrap(),
        "application/octet-stream"
    );
    for m in ["PUT", "PATCH", "DELETE"] {
        assert!(
            raw(site.addr(), m, "/api/x", &[], b"")
                .await
                .text()
                .starts_with(&format!("{m} /x\n")),
            "{m}"
        );
    }
}

#[tokio::test]
async fn upstream_status_and_headers_come_back_and_hop_by_hop_ones_do_not() {
    let (up, _) = upstream().await;
    let (_t, site) = front(&up, false).await;
    let r = raw(site.addr(), "GET", "/api/status/418", &[], b"").await;
    assert_eq!((r.status, r.text()), (418, "teapot body".into()));
    assert_eq!(r.header("x-upstream").unwrap(), "yes");
    assert!(
        r.header("keep-alive").is_none(),
        "a hop-by-hop response header was forwarded"
    );
}

#[tokio::test]
async fn hop_by_hop_and_claimed_forwarding_headers_do_not_reach_the_upstream() {
    let (up, _) = upstream().await;
    let (_t, site) = front(&up, false).await;
    let r = raw(
        site.addr(),
        "GET",
        "/api/who",
        &[
            ("Connection", "x-secret, keep-alive"),
            ("X-Secret", "named-by-connection"),
            ("Keep-Alive", "timeout=99"),
            ("Proxy-Authorization", "Basic abc"),
            ("X-Forwarded-For", "6.6.6.6"),
            ("X-Forwarded-Proto", "https"),
            ("X-Forwarded-Host", "evil.example"),
            ("Forwarded", "for=6.6.6.6"),
            ("X-Custom", "kept"),
        ],
        b"",
    )
    .await;
    let t = r.text();
    for gone in ["x-secret", "keep-alive", "proxy-authorization", "forwarded"] {
        assert!(
            seen_header(&t, gone).is_none(),
            "{gone} reached the upstream:\n{t}"
        );
    }
    assert_eq!(
        seen_header(&t, "x-custom").unwrap(),
        "kept",
        "the control: ordinary headers pass"
    );
    // What the upstream is told is what this server established, not what the client claimed.
    assert_eq!(seen_header(&t, "x-forwarded-for").unwrap(), "127.0.0.1");
    assert_eq!(seen_header(&t, "x-forwarded-proto").unwrap(), "http");
    assert_eq!(seen_header(&t, "x-forwarded-host").unwrap(), "front.test");
    assert!(
        t.matches("x-forwarded-for").count() == 1,
        "the header must appear once:\n{t}"
    );
}

#[tokio::test]
async fn dot_segments_are_refused_and_never_reach_the_upstream() {
    let (up, calls) = upstream().await;
    let (_t, site) = front(&up, false).await;
    for target in [
        "/api/../admin",
        "/api/%2e%2e/admin",
        "/api/a/%2E%2E/%2e%2e/x",
        "/api/./x",
        "/api/..%5cadmin",
    ] {
        let r = raw(site.addr(), "GET", target, &[], b"").await;
        assert_eq!(r.status, 400, "{target}");
    }
    assert_eq!(
        calls.load(Ordering::SeqCst),
        0,
        "a refused path was forwarded"
    );
    // The control: a plain path is forwarded.
    assert_eq!(
        raw(site.addr(), "GET", "/api/ok", &[], b"").await.status,
        200
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn a_path_that_only_starts_like_the_prefix_is_not_proxied() {
    let (up, calls) = upstream().await;
    let (_t, site) = front(&up, true).await;
    assert_eq!(raw(site.addr(), "GET", "/apix", &[], b"").await.status, 404);
    assert_eq!(
        raw(site.addr(), "GET", "/api2/x", &[], b"").await.status,
        404
    );
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    // Files and the proxy live side by side.
    assert_eq!(
        raw(site.addr(), "GET", "/a.txt", &[], b"").await.text(),
        "file a"
    );
    assert_eq!(
        raw(site.addr(), "GET", "/api/z", &[], b"").await.status,
        200
    );
}

#[tokio::test]
async fn an_upgrade_is_refused_out_loud_and_not_forwarded() {
    let (up, calls) = upstream().await;
    let (_t, site) = front(&up, false).await;
    let r = raw(
        site.addr(),
        "GET",
        "/api/ws",
        &[("Connection", "Upgrade"), ("Upgrade", "websocket")],
        b"",
    )
    .await;
    assert_eq!(r.status, 501);
    assert!(r.text().contains("upgrade"), "{}", r.text());
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn a_dead_upstream_is_a_502_that_does_not_say_where_it_is() {
    let (up, _) = upstream().await;
    let port = up.addr().port().to_string();
    let (_t, site) = front(&up, false).await;
    up.shutdown().await.unwrap();
    let r = raw(site.addr(), "GET", "/api/x", &[], b"").await;
    assert_eq!(r.status, 502);
    assert_eq!(r.text(), "bad gateway");
    assert!(!r.head.contains(&port) && !r.text().contains(&port));
}

#[tokio::test]
async fn an_upstream_with_a_base_path_gets_it_prepended() {
    let (up, _) = upstream().await;
    let site = Site::new("front.test")
        .bind("127.0.0.1:0")
        .proxy("/api", format!("http://{}/base/", up.addr()))
        .start()
        .await
        .unwrap();
    assert!(
        raw(site.addr(), "GET", "/api/u?q=1", &[], b"")
            .await
            .text()
            .starts_with("GET /base/u?q=1\n")
    );
}

#[tokio::test]
async fn bad_proxy_settings_are_named_at_start() {
    let e = Site::new("t")
        .bind("127.0.0.1:0")
        .proxy("api", "http://127.0.0.1:1")
        .start()
        .await
        .unwrap_err();
    assert!(
        e.to_string().contains("`/`") && e.to_string().contains(".proxy("),
        "{e}"
    );
    let e = Site::new("t")
        .bind("127.0.0.1:0")
        .proxy("/api", "http://u:pw@h")
        .start()
        .await
        .unwrap_err();
    assert!(!e.to_string().contains("pw@"), "{e}");
    // The control: nothing to serve at all is still its own error.
    assert!(
        Site::new("t")
            .bind("127.0.0.1:0")
            .start()
            .await
            .unwrap_err()
            .to_string()
            .contains("nothing to serve")
    );
}
