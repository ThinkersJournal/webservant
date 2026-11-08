// SPDX-License-Identifier: MIT OR Apache-2.0
//! `webservant serve --files ./public [--name example.org] [--bind 127.0.0.1:8080] [--cert cert.pem --key key.pem] [--dotfiles]`

use std::process::ExitCode;

const USAGE: &str = "usage: webservant serve --files DIR [--name HOST] [--bind ADDR] [--cert CERT.pem --key KEY.pem] [--dotfiles]\n\
example: webservant serve --files ./public\n\
this serves ./public at http://127.0.0.1:8080/ until Ctrl-C";

fn parse(args: Vec<String>) -> Result<webservant::Site, String> {
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
            other => {
                return Err(format!(
                    "unknown option `{other}` (a typo is not ignored).\n{USAGE}"
                ));
            }
        }
    }
    let mut site = webservant::Site::new(name).dotfiles(dot);
    if let Some(f) = files {
        site = site.files(f);
    }
    if let Some(b) = bind {
        site = site.bind(b);
    }
    match (cert, key) {
        (Some(c), Some(k)) => site = site.tls_pem_files(c, k),
        (None, None) => {}
        _ => return Err(webservant::Error::HalfCertificate.to_string()),
    }
    Ok(site)
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
    let _ = tokio::signal::ctrl_c().await;
    match running.shutdown().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("webservant: {e}");
            ExitCode::FAILURE
        }
    }
}
