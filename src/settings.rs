// SPDX-License-Identifier: MIT OR Apache-2.0
//! The whole configuration of a site as plain data, so a settings system (ConfigurAnt fills any serde struct by dotted key) or a file in any
//! serde format can drive it with no code: `Site::from_settings(&settings)`. The command line builds one of these too, so there is a single
//! path from configuration to a running site.
//!
//! A misspelled key is an error that names it (`deny_unknown_fields`): a typo in `contact` must not silently mean "no contact".

use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{Acme, Error, Result, Site};

/// Everything a site needs. Only `name` is required, and one of `files` or a `proxy` entry; every other field has the default the builder has.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Settings {
    /// The host name the site is for (and the name its certificate is requested for).
    pub name: String,
    /// The directory to serve.
    pub files: Option<PathBuf>,
    /// Where the HTTPS (or only) listener binds. Default: `127.0.0.1:8080`, or `0.0.0.0:443` with `acme`.
    pub bind: Option<String>,
    /// With `acme`: the plain-HTTP listener for the challenge and the redirect. Default `0.0.0.0:80`.
    pub http_bind: Option<String>,
    /// The web library, by HTTPAnt's id (default hyper).
    pub adapter: Option<String>,
    /// Serve names starting with a dot.
    pub dotfiles: bool,
    /// A certificate and key you already have.
    pub tls: Option<TlsFiles>,
    /// Automatic certificates.
    pub acme: Option<AcmeSettings>,
    /// Paths forwarded to apps, checked before the files.
    pub proxy: Vec<ProxyRoute>,
}

/// A PEM certificate chain (leaf first) and its private key, as file paths.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct TlsFiles {
    /// The certificate chain.
    pub cert: Option<PathBuf>,
    /// The private key.
    pub key: Option<PathBuf>,
}

/// Automatic certificates; see [`Acme`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct AcmeSettings {
    /// The contact address the certificate authority writes to.
    pub contact: String,
    /// Use Let's Encrypt staging (do the first run this way).
    pub staging: bool,
    /// Another certificate authority's ACME directory URL.
    pub directory: Option<String>,
    /// A PEM file of roots to trust when talking to the CA (a private or test CA).
    pub ca_root_pem_file: Option<PathBuf>,
    /// Where the account and certificates (private keys) are kept.
    pub store: Option<PathBuf>,
    /// More names covered by the same certificate.
    pub also_names: Vec<String>,
    /// How often, in seconds, the renewal loop looks at the certificate (default twelve hours).
    pub check_every_secs: Option<u64>,
}

/// One forwarded path.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct ProxyRoute {
    /// The path prefix, such as `/api`.
    pub prefix: String,
    /// The app, such as `http://127.0.0.1:3000`.
    pub upstream: String,
}

fn bad(what: &str, why: &str) -> Error {
    Error::Settings(format!("`{what}` {why}"))
}

impl Site {
    /// A site from settings. Everything wrong with the settings is reported here or by [`Site::start`], before anything listens.
    pub fn from_settings(s: &Settings) -> Result<Self> {
        if s.name.trim().is_empty() {
            return Err(bad(
                "name",
                "is empty: give the host name the site is for, for example `example.org`",
            ));
        }
        let mut site = Site::new(s.name.trim()).dotfiles(s.dotfiles);
        if let Some(f) = &s.files {
            site = site.files(f);
        }
        if let Some(b) = &s.bind {
            site = site.bind(b);
        }
        if let Some(b) = &s.http_bind {
            site = site.http_bind(b);
        }
        if let Some(a) = &s.adapter {
            site = site.adapter(a);
        }
        for r in &s.proxy {
            site = site.proxy(&r.prefix, &r.upstream);
        }
        if let Some(t) = &s.tls {
            match (&t.cert, &t.key) {
                (Some(c), Some(k)) => site = site.tls_pem_files(c, k),
                (None, None) => {}
                _ => return Err(Error::HalfCertificate),
            }
        }
        if let Some(a) = &s.acme {
            if a.contact.trim().is_empty() {
                return Err(bad(
                    "acme.contact",
                    "is empty: give an address the certificate authority can write to, for example `you@example.org`",
                ));
            }
            let mut acme = Acme::new(a.contact.trim());
            if a.staging {
                acme = acme.staging();
            }
            if let Some(d) = &a.directory {
                if a.staging {
                    return Err(bad(
                        "acme.staging",
                        "and `acme.directory` were both given: staging is a directory, so use one of them",
                    ));
                }
                acme = acme.directory(d);
            }
            if let Some(p) = &a.ca_root_pem_file {
                acme = acme.ca_root_pem_file(p);
            }
            if let Some(d) = &a.store {
                acme = acme.store(d);
            }
            for n in &a.also_names {
                acme = acme.also_name(n);
            }
            if let Some(secs) = a.check_every_secs {
                if secs == 0 {
                    return Err(bad(
                        "acme.check_every_secs",
                        "is 0: use a number of seconds of 1 or more, or leave it out for twelve hours",
                    ));
                }
                acme = acme.check_every(Duration::from_secs(secs));
            }
            site = site.acme(acme);
        }
        Ok(site)
    }
}
