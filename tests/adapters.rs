// SPDX-License-Identifier: MIT OR Apache-2.0
//! `.adapter(id)` picks the web library. With the feature on, the site serves a file on that library; with it off, start refuses and the
//! error names the feature. CI builds this file once per feature (`--features <id>`) and once with none, so every id is exercised both ways.

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use webservant::Site;

const IDS: [&str; 5] = ["axum", "actix-web", "poem", "salvo", "warp"];

fn enabled(id: &str) -> bool {
    match id {
        "axum" => cfg!(feature = "axum"),
        "actix-web" => cfg!(feature = "actix-web"),
        "poem" => cfg!(feature = "poem"),
        "salvo" => cfg!(feature = "salvo"),
        "warp" => cfg!(feature = "warp"),
        _ => unreachable!(),
    }
}

async fn get_hello(site: &webservant::RunningSite) -> String {
    let mut s = TcpStream::connect(site.addr()).await.unwrap();
    s.write_all(b"GET /hello.txt HTTP/1.1\r\nHost: test\r\nConnection: close\r\n\r\n")
        .await
        .unwrap();
    let mut buf = Vec::new();
    s.read_to_end(&mut buf).await.unwrap();
    String::from_utf8_lossy(&buf).into_owned()
}

#[tokio::test]
async fn every_adapter_serves_a_file_when_built_in_and_refuses_with_a_named_feature_when_not() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("hello.txt"), "hello adapter").unwrap();
    let mut served = 0;
    for id in IDS {
        let site = Site::new("localhost")
            .files(dir.path())
            .bind("127.0.0.1:0")
            .adapter(id)
            .start()
            .await;
        if enabled(id) {
            let site = site.unwrap_or_else(|e| panic!("{id} is built in but did not start: {e}"));
            let reply = get_hello(&site).await;
            assert!(reply.starts_with("HTTP/1.1 200"), "{id}: {reply}");
            assert!(reply.ends_with("hello adapter"), "{id}: {reply}");
            site.shutdown().await.unwrap();
            served += 1;
        } else {
            let err = match site {
                Ok(_) => panic!("{id} is not built in but started"),
                Err(e) => e.to_string(),
            };
            assert!(
                err.contains(id) && err.to_lowercase().contains("feature"),
                "the error for {id} must name the feature to turn on: {err}"
            );
        }
    }
    assert_eq!(served, IDS.iter().filter(|i| enabled(i)).count());
}
