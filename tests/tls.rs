// SPDX-License-Identifier: MIT OR Apache-2.0
//! HTTPS from a certificate and key on disk, called by HTTPAnt's own client with only that certificate trusted.

use std::fs;

use httpant::Client;
use webservant::Site;

fn pair(dir: &std::path::Path, name: &str) -> (String, std::path::PathBuf, std::path::PathBuf) {
    let ck = rcgen::generate_simple_self_signed(vec![name.to_owned()]).unwrap();
    let cert_pem = ck.cert.pem();
    let (c, k) = (dir.join("cert.pem"), dir.join("key.pem"));
    fs::write(&c, &cert_pem).unwrap();
    fs::write(&k, ck.signing_key.serialize_pem()).unwrap();
    (cert_pem, c, k)
}

#[tokio::test]
async fn serves_https_from_a_certificate_pair() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("public");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("index.html"), "secure hello").unwrap();
    let (cert_pem, c, k) = pair(tmp.path(), "localhost");
    let site = Site::new("localhost")
        .files(&root)
        .bind("127.0.0.1:0")
        .tls_pem_files(&c, &k)
        .start()
        .await
        .unwrap();
    let url = format!("https://localhost:{}/", site.addr().port());
    assert!(site.url().starts_with("https://"));

    let trusting = Client::builder().add_root_pem(&cert_pem).build().unwrap();
    assert_eq!(
        trusting
            .get(&url)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap(),
        "secure hello"
    );

    // The control: a client that does not trust this certificate refuses it (the certificate is really being checked).
    let plain = Client::builder().build().unwrap();
    assert!(plain.get(&url).send().await.is_err());
    site.shutdown().await.unwrap();
}

#[tokio::test]
async fn a_key_that_does_not_match_is_refused_without_printing_it() {
    let tmp = tempfile::tempdir().unwrap();
    let (_p, c, _k) = pair(tmp.path(), "localhost");
    let other = tempfile::tempdir().unwrap();
    let (_p2, _c2, k2) = pair(other.path(), "localhost");
    let key_text = fs::read_to_string(&k2).unwrap();
    let e = Site::new("localhost")
        .files(tmp.path())
        .bind("127.0.0.1:0")
        .tls_pem_files(&c, &k2)
        .start()
        .await
        .unwrap_err();
    let msg = e.to_string();
    assert!(msg.contains("does not match"), "{msg}");
    let body: String = key_text
        .lines()
        .filter(|l| !l.starts_with("-----"))
        .collect();
    assert!(
        !msg.contains(&body) && !format!("{e:?}").contains(&body),
        "the key text reached an error"
    );
}
