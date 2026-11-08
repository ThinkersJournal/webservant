// SPDX-License-Identifier: MIT OR Apache-2.0
//! The settings struct: what a settings system or a file in a serde format hands us becomes the same site the builder makes, and a mistake in it
//! is named. (JSON stands in for the format here; the struct is plain serde. It has NOT been tried with ConfigurAnt itself.)

use std::fs;

use webservant::{Settings, Site};

fn parse(json: &str) -> Result<Settings, serde_json::Error> {
    serde_json::from_str(json)
}

#[test]
fn a_full_settings_document_round_trips() {
    let json = r#"{
        "name": "example.org",
        "files": "./public",
        "bind": "0.0.0.0:443",
        "http_bind": "0.0.0.0:80",
        "adapter": "hyper",
        "dotfiles": true,
        "acme": {
            "contact": "me@example.org",
            "staging": true,
            "store": "/var/lib/webservant",
            "also_names": ["www.example.org"],
            "check_every_secs": 3600
        },
        "proxy": [{"prefix": "/api", "upstream": "http://127.0.0.1:3000"}]
    }"#;
    let s = parse(json).unwrap();
    assert_eq!(s.name, "example.org");
    assert_eq!(s.proxy[0].prefix, "/api");
    assert_eq!(s.acme.as_ref().unwrap().check_every_secs, Some(3600));
    let again: Settings = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
    assert_eq!(s, again);
}

#[test]
fn a_misspelled_key_is_an_error_that_names_it() {
    for (json, key) in [
        (r#"{"name":"a","fils":"x"}"#, "fils"),
        (r#"{"name":"a","acme":{"contcat":"me@a.org"}}"#, "contcat"),
        (
            r#"{"name":"a","proxy":[{"prefx":"/a","upstream":"http://h"}]}"#,
            "prefx",
        ),
        (r#"{"name":"a","tls":{"certt":"c"}}"#, "certt"),
    ] {
        let e = parse(json).unwrap_err().to_string();
        assert!(e.contains(key), "{json}: {e}");
    }
    // The control: the same documents spelled right parse.
    assert!(parse(r#"{"name":"a","files":"x"}"#).is_ok());
    assert!(parse(r#"{"name":"a","acme":{"contact":"me@a.org"}}"#).is_ok());
}

#[test]
fn only_the_name_is_needed_and_the_rest_defaults() {
    let s = parse(r#"{"name":"a"}"#).unwrap();
    assert_eq!(
        s,
        Settings {
            name: "a".into(),
            ..Default::default()
        }
    );
    assert!(!s.dotfiles && s.files.is_none() && s.acme.is_none() && s.proxy.is_empty());
}

fn problem(json: &str) -> String {
    Site::from_settings(&parse(json).unwrap())
        .unwrap_err()
        .to_string()
}

#[test]
fn settings_that_cannot_work_are_named() {
    assert!(problem(r#"{"name":"  "}"#).contains("`name`"));
    assert!(problem(r#"{"name":"a","tls":{"cert":"c.pem"}}"#).contains("without its key"));
    assert!(problem(r#"{"name":"a","acme":{"contact":""}}"#).contains("acme.contact"));
    assert!(problem(r#"{"name":"a","acme":{"contact":"m@a.org","staging":true,"directory":"https://x/dir"}}"#).contains("both given"));
    assert!(
        problem(r#"{"name":"a","acme":{"contact":"m@a.org","check_every_secs":0}}"#)
            .contains("check_every_secs")
    );
}

#[tokio::test]
async fn a_site_from_settings_serves_like_one_from_the_builder() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("index.html"), "from settings").unwrap();
    let s = Settings {
        name: "localhost".into(),
        files: Some(tmp.path().to_owned()),
        bind: Some("127.0.0.1:0".into()),
        ..Default::default()
    };
    let site = Site::from_settings(&s).unwrap().start().await.unwrap();
    let text = httpant::Client::new()
        .get(site.url())
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert_eq!(text, "from settings");
    site.shutdown().await.unwrap();

    // Nothing to serve (no files, no proxy) is still the same error as with the builder.
    let none = Settings {
        name: "localhost".into(),
        bind: Some("127.0.0.1:0".into()),
        ..Default::default()
    };
    let e = Site::from_settings(&none)
        .unwrap()
        .start()
        .await
        .unwrap_err();
    assert!(e.to_string().contains("nothing to serve"), "{e}");
}

#[tokio::test]
async fn proxy_and_dotfile_settings_reach_the_site() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join(".hidden"), "dot").unwrap();
    let s: Settings = serde_json::from_str(&format!(
        r#"{{"name":"localhost","files":{},"bind":"127.0.0.1:0","dotfiles":true,
            "proxy":[{{"prefix":"/api","upstream":"http://127.0.0.1:1"}}]}}"#,
        serde_json::to_string(tmp.path()).unwrap()
    ))
    .unwrap();
    let site = Site::from_settings(&s).unwrap().start().await.unwrap();
    let c = httpant::Client::new();
    // dotfiles = true is honoured.
    let url = site.url();
    assert_eq!(
        c.get(format!("{url}.hidden"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap(),
        "dot"
    );
    // The proxy route is registered: a dead upstream is a 502 (without it this path would be a 404 from the files).
    assert_eq!(
        c.get(format!("{url}api/x"))
            .send()
            .await
            .unwrap()
            .status()
            .as_u16(),
        502
    );
    site.shutdown().await.unwrap();
}
