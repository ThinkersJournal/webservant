// SPDX-License-Identifier: MIT OR Apache-2.0
//! `webservant serve --files ./public [--name example.org] [--bind 127.0.0.1:8080] [--cert cert.pem --key key.pem] [--dotfiles]`

use std::process::ExitCode;

const USAGE: &str = "usage: webservant serve --files DIR [--name HOST] [--bind ADDR] [--cert CERT.pem --key KEY.pem] [--dotfiles]\n\
automatic HTTPS: webservant serve --files DIR --name HOST --acme you@example.org [--staging] [--also-name HOST] [--acme-store DIR] [--http-bind ADDR]\n\
forward a path to an app: webservant serve --files DIR --proxy /api=http://127.0.0.1:3000   (--proxy may repeat; --files is then optional)\n\
example: webservant serve --files ./public\n\
this serves ./public at http://127.0.0.1:8080/ until Ctrl-C";

fn settings_from_args(args: Vec<String>) -> Result<webservant::Settings, String> {
    let mut it = args.into_iter();
    match it.next().as_deref() {
        Some("serve") => {}
        Some("--help" | "-h") | None => return Err(USAGE.to_owned()),
        Some(other) => {
            return Err(format!(
                "unknown command `{other}`: the only command is `serve`.\n{USAGE}"
            ));
        }
    }
    let (mut name, mut files, mut bind, mut cert, mut key, mut dot) =
        ("localhost".to_owned(), None, None, None, None, false);
    let mut proxies: Vec<(String, String)> = Vec::new();
    let (mut acme, mut staging, mut store, mut http_bind, mut also) =
        (None, false, None, None, Vec::new());
    while let Some(flag) = it.next() {
        let mut value = |what: &str| {
            it.next()
                .ok_or_else(|| format!("`{what}` needs a value.\n{USAGE}"))
        };
        match flag.as_str() {
            "--name" => name = value("--name")?,
            "--files" => files = Some(value("--files")?),
            "--bind" => bind = Some(value("--bind")?),
            "--cert" => cert = Some(value("--cert")?),
            "--key" => key = Some(value("--key")?),
            "--dotfiles" => dot = true,
            "--proxy" => {
                let v = value("--proxy")?;
                let (prefix, upstream) = v.split_once('=').ok_or_else(|| {
                    format!("`--proxy` takes PREFIX=UPSTREAM, for example `--proxy /api=http://127.0.0.1:3000`.\n{USAGE}")
                })?;
                proxies.push((prefix.to_owned(), upstream.to_owned()));
            }
            "--acme" => acme = Some(value("--acme")?),
            "--staging" => staging = true,
            "--acme-store" => store = Some(value("--acme-store")?),
            "--http-bind" => http_bind = Some(value("--http-bind")?),
            "--also-name" => also.push(value("--also-name")?),
            other => {
                return Err(format!(
                    "unknown option `{other}` (a typo is not ignored).\n{USAGE}"
                ));
            }
        }
    }
    if acme.is_none() && (staging || store.is_some() || !also.is_empty()) {
        return Err(format!(
            "`--staging`, `--acme-store` and `--also-name` only apply with `--acme you@example.org`.
{USAGE}"
        ));
    }
    let tls = match (cert, key) {
        (Some(c), Some(k)) => Some(webservant::TlsFiles {
            cert: Some(c.into()),
            key: Some(k.into()),
        }),
        (None, None) => None,
        _ => return Err(webservant::Error::HalfCertificate.to_string()),
    };
    Ok(webservant::Settings {
        name,
        files: files.map(Into::into),
        bind,
        http_bind,
        adapter: None,
        dotfiles: dot,
        tls,
        acme: acme.map(|contact| webservant::AcmeSettings {
            contact,
            staging,
            store: store.map(Into::into),
            also_names: also,
            ..Default::default()
        }),
        proxy: proxies
            .into_iter()
            .map(|(prefix, upstream)| webservant::ProxyRoute { prefix, upstream })
            .collect(),
    })
}

fn parse(args: Vec<String>) -> Result<webservant::Site, String> {
    let settings = settings_from_args(args)?;
    webservant::Site::from_settings(&settings).map_err(|e| e.to_string())
}

#[tokio::main]
async fn main() -> ExitCode {
    let site = match parse(std::env::args().skip(1).collect()) {
        Ok(s) => s,
        Err(msg) => {
            eprintln!("{msg}");
            return ExitCode::from(2);
        }
    };
    let running = match site.start().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("webservant: {e}");
            return ExitCode::FAILURE;
        }
    };
    println!("serving {}  (Ctrl-C to stop)", running.url());
    webservant::shutdown_signal().await;
    match running.shutdown().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("webservant: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn flags_become_settings() {
        let s = settings_from_args(args(&[
            "serve",
            "--files",
            "pub",
            "--name",
            "example.org",
            "--acme",
            "me@example.org",
            "--staging",
            "--also-name",
            "www.example.org",
            "--proxy",
            "/api=http://127.0.0.1:3000",
            "--dotfiles",
        ]))
        .unwrap();
        assert_eq!(s.name, "example.org");
        assert_eq!(s.files.as_deref(), Some(std::path::Path::new("pub")));
        assert!(s.dotfiles);
        let a = s.acme.unwrap();
        assert_eq!(
            (a.contact.as_str(), a.staging, a.also_names.as_slice()),
            ("me@example.org", true, &["www.example.org".to_owned()][..])
        );
        assert_eq!(s.proxy.len(), 1);
        assert_eq!(
            (s.proxy[0].prefix.as_str(), s.proxy[0].upstream.as_str()),
            ("/api", "http://127.0.0.1:3000")
        );
    }

    #[test]
    fn mistakes_are_named() {
        for bad in [
            &["serve", "--flies", "x"][..],
            &["serve", "--staging"][..],
            &["serve", "--cert", "c.pem"][..],
            &["serve", "--proxy", "nope"][..],
            &["serve", "--files"][..],
            &["frobnicate"][..],
        ] {
            assert!(settings_from_args(args(bad)).is_err(), "{bad:?}");
        }
    }
}
