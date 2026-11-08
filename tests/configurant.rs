// SPDX-License-Identifier: MIT OR Apache-2.0
//! `Settings` filled by ConfigurAnt (published 0.6.3) through the schema in `configurant.toml`: layering, validation, and binding to the
//! struct. The same document read directly with serde is the cross-check, so a key the schema forgot shows up as a difference.

use configurant::{Loader, Schema};
use webservant::{AcmeSettings, ProxyRoute, Settings, Site};

const SCHEMA: &str = include_str!("../configurant.toml");

const FILE: &str = r#"
name = "example.org"
files = "./public"
bind = "0.0.0.0:443"
http_bind = "0.0.0.0:80"
dotfiles = true
max_body_bytes = 10485760

[acme]
contact = "you@example.org"
staging = true
store = "/var/lib/webservant"
also_names = ["www.example.org"]
check_every_secs = 3600

[tls]
cert = "c.pem"
key = "k.pem"

[[proxy]]
prefix = "/api"
upstream = "http://127.0.0.1:3000"
connect_timeout_secs = 2
response_timeout_secs = 30
idle_timeout_secs = 45
"#;

fn settings_from(file: &str) -> Settings {
    let schema = Schema::parse(SCHEMA).expect("the shipped schema parses");
    let config = Loader::new(&schema)
        .file("webservant.toml", file)
        .build()
        .unwrap_or_else(|e| panic!("{e:?}"));
    config.deserialize::<Settings>().unwrap()
}

#[test]
fn configurant_fills_settings_the_same_as_reading_the_file_directly() {
    let via_configurant = settings_from(FILE);
    let direct: Settings = toml::from_str(FILE).unwrap();
    assert_eq!(
        via_configurant, direct,
        "ConfigurAnt and serde disagree about the same document"
    );
    // Spot checks, so the comparison above cannot pass by both being empty.
    assert_eq!(via_configurant.name, "example.org");
    assert_eq!(
        via_configurant.acme.as_ref().unwrap().also_names,
        vec!["www.example.org".to_string()]
    );
    assert_eq!(
        via_configurant.acme.as_ref().unwrap().check_every_secs,
        Some(3600)
    );
    assert_eq!(
        via_configurant.proxy,
        vec![ProxyRoute {
            prefix: "/api".into(),
            upstream: "http://127.0.0.1:3000".into(),
            connect_timeout_secs: Some(2),
            response_timeout_secs: Some(30),
            idle_timeout_secs: Some(45),
        }]
    );
    assert!(via_configurant.dotfiles);
    assert_eq!(via_configurant.max_body_bytes, Some(10_485_760));
}

#[tokio::test]
async fn with_nothing_set_under_acme_or_tls_the_site_is_plain_http() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("index.html"), "plain").unwrap();
    let doc = format!(
        "name = \"localhost\"
files = {}
bind = \"127.0.0.1:0\"
",
        serde_json::to_string(tmp.path()).unwrap()
    );
    let s = settings_from(&doc);
    // ConfigurAnt hands over EMPTY tls and acme tables (it builds a table for every declared prefix), not None: that is what this guards.
    assert!(
        s.acme
            .as_ref()
            .is_some_and(|a| *a == AcmeSettings::default()),
        "{s:?}"
    );
    assert!(s.tls.is_some() && s.proxy.is_empty());
    let site = Site::from_settings(&s).unwrap().start().await.unwrap();
    assert!(
        site.url().starts_with("http://"),
        "an empty acme table must not turn automatic certificates on"
    );
    let text = httpant::Client::new()
        .get(site.url())
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert_eq!(text, "plain");
    site.shutdown().await.unwrap();
}

#[test]
fn one_acme_key_turns_automatic_certificates_on() {
    let s =
        settings_from("name = \"example.org\"\nfiles = \"./p\"\n[acme]\ncontact = \"a@b.org\"\n");
    assert_eq!(
        s.acme,
        Some(AcmeSettings {
            contact: "a@b.org".into(),
            ..Default::default()
        })
    );
}

#[test]
fn configurant_validates_before_webservant_sees_anything() {
    let schema = Schema::parse(SCHEMA).unwrap();
    // A missing required name, an out-of-range number and a misspelled key are all refused by ConfigurAnt, with named errors.
    for (doc, mention) in [
        ("files = \"x\"\n", "name"),
        (
            "name = \"a\"\n[acme]\ncheck_every_secs = 0\n",
            "check_every_secs",
        ),
        ("name = \"a\"\nfils = \"x\"\n", "fils"),
    ] {
        let err = Loader::new(&schema)
            .file("w.toml", doc)
            .build()
            .err()
            .unwrap_or_else(|| panic!("{doc} was accepted"));
        assert!(
            format!("{err:?}").contains(mention) || err.to_string().contains(mention),
            "{doc}: {err:?}"
        );
    }
}
