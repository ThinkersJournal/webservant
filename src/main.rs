// SPDX-License-Identifier: MIT OR Apache-2.0
//! `webservant serve --files ./public [--name example.org] [--bind 127.0.0.1:8080] [--cert cert.pem --key key.pem] [--dotfiles]`

use std::process::ExitCode;

const USAGE: &str = "usage: webservant serve --files DIR [--name HOST] [--bind ADDR] [--cert CERT.pem --key KEY.pem] [--dotfiles] [--max-body BYTES]\n\
automatic HTTPS: webservant serve --files DIR --name HOST --acme you@example.org [--staging] [--also-name HOST] [--acme-store DIR] [--http-bind ADDR] [--acme-directory URL] [--acme-ca-root PEMFILE]\n\
forward a path to an app: webservant serve --files DIR --proxy /api=http://127.0.0.1:3000   (--proxy may repeat; --files is then optional)\n\
will automatic HTTPS work here? (one staging issuance, nothing kept): webservant check --name HOST --acme you@example.org --files DIR\n\
example: webservant serve --files ./public\n\
this serves ./public at http://127.0.0.1:8080/ until Ctrl-C";

/// Every option the command line knows, for "did you mean".
const OPTIONS: [&str; 19] = [
    "--name",
    "--files",
    "--bind",
    "--cert",
    "--key",
    "--dotfiles",
    "--proxy",
    "--max-body",
    "--acme",
    "--staging",
    "--acme-store",
    "--acme-directory",
    "--acme-ca-root",
    "--http-bind",
    "--also-name",
    "--help",
    "--version",
    "-h",
    "-V",
];

/// The known option closest to `typed`, if it is within two edits.
fn suggestion(typed: &str) -> Option<&'static str> {
    fn distance(a: &str, b: &str) -> usize {
        let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
        let mut prev: Vec<usize> = (0..=b.len()).collect();
        for (i, ca) in a.iter().enumerate() {
            let mut cur = vec![i + 1];
            for (j, cb) in b.iter().enumerate() {
                cur.push(
                    (prev[j] + usize::from(ca != cb))
                        .min(prev[j + 1] + 1)
                        .min(cur[j] + 1),
                );
            }
            prev = cur;
        }
        prev[b.len()]
    }
    OPTIONS
        .iter()
        .map(|o| (distance(typed, o), *o))
        .filter(|(d, _)| *d <= 2)
        .min_by_key(|(d, _)| *d)
        .map(|(_, o)| o)
}

fn settings_from_args(args: Vec<String>) -> Result<webservant::Settings, String> {
    let mut it = args.into_iter();
    match it.next().as_deref() {
        Some("serve" | "check") => {}
        Some("--help" | "-h") | None => return Err(USAGE.to_owned()),
        Some(other) => {
            return Err(format!(
                "unknown command `{other}`: the commands are `serve` and `check`.\n{USAGE}"
            ));
        }
    }
    let (mut name, mut files, mut bind, mut cert, mut key, mut dot) =
        ("localhost".to_owned(), None, None, None, None, false);
    let mut proxies: Vec<(String, String)> = Vec::new();
    let mut max_body: Option<u64> = None;
    let (mut directory, mut ca_root): (Option<String>, Option<String>) = (None, None);
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
            "--max-body" => {
                let v = value("--max-body")?;
                max_body = Some(v.parse::<u64>().map_err(|_| {
                    format!(
                        "`--max-body` takes a number of bytes, for example `--max-body 10485760`.
{USAGE}"
                    )
                })?);
            }
            "--proxy" => {
                let v = value("--proxy")?;
                let (prefix, upstream) = v.split_once('=').ok_or_else(|| {
                    format!("`--proxy` takes PREFIX=UPSTREAM, for example `--proxy /api=http://127.0.0.1:3000`.\n{USAGE}")
                })?;
                proxies.push((prefix.to_owned(), upstream.to_owned()));
            }
            "--acme" => acme = Some(value("--acme")?),
            "--staging" => staging = true,
            "--acme-directory" => directory = Some(value("--acme-directory")?),
            "--acme-ca-root" => ca_root = Some(value("--acme-ca-root")?),
            "--acme-store" => store = Some(value("--acme-store")?),
            "--http-bind" => http_bind = Some(value("--http-bind")?),
            "--also-name" => also.push(value("--also-name")?),
            other => {
                let hint = suggestion(other)
                    .map(|s| format!(" Did you mean `{s}`?"))
                    .unwrap_or_default();
                return Err(format!(
                    "unknown option `{other}` (a typo is not ignored).{hint} Run `webservant --help` to see every option."
                ));
            }
        }
    }
    if acme.is_none()
        && (staging
            || store.is_some()
            || !also.is_empty()
            || directory.is_some()
            || ca_root.is_some())
    {
        return Err(format!(
            "`--staging`, `--acme-store`, `--also-name`, `--acme-directory` and `--acme-ca-root` only apply with `--acme you@example.org`.
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
        max_body_bytes: max_body,
        tls,
        acme: acme.map(|contact| webservant::AcmeSettings {
            contact,
            staging,
            directory,
            ca_root_pem_file: ca_root.map(Into::into),
            store: store.map(Into::into),
            also_names: also,
            ..Default::default()
        }),
        proxy: proxies
            .into_iter()
            .map(|(prefix, upstream)| webservant::ProxyRoute {
                prefix,
                upstream,
                ..Default::default()
            })
            .collect(),
    })
}

fn parse(args: Vec<String>) -> Result<webservant::Site, String> {
    let settings = settings_from_args(args)?;
    webservant::Site::from_settings(&settings).map_err(|e| e.to_string())
}

/// `webservant check`: one real issuance against staging (or the configured test CA), then clean up. Exit 0 if a certificate was issued.
async fn run_check(site: webservant::Site) -> ExitCode {
    match site.check_reachable().await {
        Ok(r) if r.is_reachable() => {
            println!(
                "OK: {} issued a certificate for {}. Nothing was kept.",
                r.directory,
                r.names.join(", ")
            );
            ExitCode::SUCCESS
        }
        Ok(r) => {
            eprintln!(
                "NOT REACHABLE ({}): {}",
                r.directory,
                r.failure
                    .unwrap_or_else(|| "no certificate was issued".into())
            );
            ExitCode::FAILURE
        }
        Err(e) => {
            eprintln!("webservant: {e}");
            ExitCode::FAILURE
        }
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if matches!(args.first().map(String::as_str), Some("--version" | "-V")) {
        println!("webservant {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    if matches!(args.first().map(String::as_str), Some("help"))
        || args.iter().any(|a| a == "--help" || a == "-h")
    {
        println!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    let check = args.first().map(String::as_str) == Some("check");
    let site = match parse(args) {
        Ok(s) => s,
        Err(msg) => {
            eprintln!("{msg}");
            return ExitCode::from(2);
        }
    };
    if check {
        return run_check(site).await;
    }
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

    /// Every `webservant ...` command line shown in a code block of the README and the quickstart must be accepted by the real argument parser,
    /// so a page cannot teach a flag that does not exist.
    #[test]
    fn the_documented_commands_parse() {
        let mut seen = 0;
        for (name, text) in [
            ("README.md", include_str!("../README.md")),
            ("docs/QUICKSTART.md", include_str!("../docs/QUICKSTART.md")),
        ] {
            let mut in_block = false;
            for line in text.lines() {
                if line.trim_start().starts_with("```") {
                    in_block = !in_block;
                    continue;
                }
                let line = line.trim();
                if in_block && line.starts_with("webservant ") && !line.contains("...") {
                    let args: Vec<String> =
                        line.split_whitespace().skip(1).map(str::to_owned).collect();
                    settings_from_args(args)
                        .unwrap_or_else(|e| panic!("{name}: `{line}` does not parse: {e}"));
                    seen += 1;
                }
            }
        }
        assert!(
            seen >= 5,
            "found only {seen} documented commands: the extraction broke"
        );
    }

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
    fn a_typo_gets_a_suggestion_and_no_wall_of_usage() {
        let e = settings_from_args(args(&["serve", "--flies", "x"])).unwrap_err();
        assert!(
            e.contains("`--flies`") && e.contains("Did you mean `--files`?"),
            "{e}"
        );
        assert!(
            e.lines().count() == 1,
            "the error must be one line, not the usage wall: {e}"
        );
        // Something that resembles nothing gets no guess.
        let e = settings_from_args(args(&["serve", "--zzzzzzzz", "x"])).unwrap_err();
        assert!(!e.contains("Did you mean"), "{e}");
        assert_eq!(suggestion("--acm"), Some("--acme"));
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
