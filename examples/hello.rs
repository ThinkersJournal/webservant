// SPDX-License-Identifier: MIT OR Apache-2.0
//! Serves a directory, fetches a file from it with HTTPAnt's client, checks it, and stops. Exits 0 when it worked.
use httpant::Client;
use webservant::Site;

#[tokio::main]
async fn main() -> webservant::Result<()> {
    let dir = std::env::temp_dir().join("webservant-hello");
    std::fs::create_dir_all(&dir).expect("a temp directory");
    std::fs::write(dir.join("hello.txt"), "hello").expect("a file in it");

    let site = Site::new("localhost")
        .files(&dir)
        .bind("127.0.0.1:0")
        .start()
        .await?;
    let text = Client::new()
        .get(format!("{}hello.txt", site.url()))
        .send()
        .await?
        .text()
        .await?;
    assert_eq!(text, "hello");
    site.shutdown().await
}
