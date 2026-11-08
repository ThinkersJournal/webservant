// SPDX-License-Identifier: MIT OR Apache-2.0
//! # WebServAnt
//!
//! A web server that runs on the Rust web library you choose, because it is built on [HTTPAnt](https://crates.io/crates/httpant).
//! It serves a directory of static files over plain HTTP, over HTTPS from a certificate and key you already have, or over HTTPS with a
//! certificate it gets and renews itself ([`Acme`]). A reverse proxy is on the roadmap (`ROADMAP.md`), not in this release.
//!
//! ```no_run
//! # async fn run() -> webservant::Result<()> {
//! webservant::Site::new("example.org")
//!     .files("./public")
//!     .serve()
//!     .await
//! # }
//! ```
//!
//! That serves `./public` on `127.0.0.1:8080` until Ctrl-C. Nothing is exposed to the network until you ask: use `.bind("0.0.0.0:80")`, or
//! `.acme(Acme::new("me@example.org"))`, which listens on ports 80 and 443 and gets a certificate for the site's name.

pub mod acme;
mod error;
mod files;
mod proxy;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use http::Method;
use httpant::server::{FixedCert, HttpsRedirect, ServerCert, TlsAcceptor};
use httpant::{Request, RunningServer, Server};

pub use acme::{
    AcmeClient, AcmeError, AcmeEvent, CertInfo, CertificateStatus, ChallengeKind, Challenges,
    Issued,
};
pub use error::{Error, Result};

/// The address used when you do not choose one: this machine only.
pub const DEFAULT_BIND: &str = "127.0.0.1:8080";

/// Automatic certificates. `Acme::new("me@example.org")` asks Let's Encrypt (production) for a certificate for the site's name, answers the
/// HTTP-01 challenge on port 80, serves HTTPS on port 443 and renews before expiry. **Try `.staging()` first**: production has rate limits that
/// a mistake in DNS or ports can use up.
pub struct Acme {
    contact: Option<String>,
    directory: String,
    root_pem: Option<PathBuf>,
    store: Option<PathBuf>,
    kind: ChallengeKind,
    extra_names: Vec<String>,
    client: Option<Box<dyn AcmeClient>>,
    sink: Option<acme::EventSink>,
    check_every: Duration,
}

impl std::fmt::Debug for Acme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Acme")
            .field("directory", &self.directory)
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

impl Acme {
    /// Certificates for the site's name, with this contact address (the CA writes to it about expiry and problems).
    pub fn new(contact: impl Into<String>) -> Self {
        Self {
            contact: Some(contact.into()),
            directory: acme::LETS_ENCRYPT_PRODUCTION.to_owned(),
            root_pem: None,
            store: None,
            kind: ChallengeKind::Http01,
            extra_names: Vec::new(),
            client: None,
            sink: None,
            check_every: Duration::from_secs(12 * 3600),
        }
    }

    /// Use Let's Encrypt's staging CA: certificates it issues are not trusted by browsers, and its rate limits are generous.
    pub fn staging(mut self) -> Self {
        self.directory = acme::LETS_ENCRYPT_STAGING.to_owned();
        self
    }

    /// Use another certificate authority, by its ACME directory URL.
    pub fn directory(mut self, url: impl Into<String>) -> Self {
        self.directory = url.into();
        self
    }

    /// Trust this PEM file of root certificates when talking to the CA (a private CA or a test CA such as Pebble).
    pub fn ca_root_pem_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.root_pem = Some(path.into());
        self
    }

    /// Keep the account and certificates here (default: your user data directory, under `webservant`). Contains private keys.
    pub fn store(mut self, dir: impl Into<PathBuf>) -> Self {
        self.store = Some(dir.into());
        self
    }

    /// Cover one more name with the same certificate.
    pub fn also_name(mut self, name: impl Into<String>) -> Self {
        self.extra_names.push(name.into());
        self
    }

    /// Receive [`AcmeEvent`]s (issued, loaded, renewal failed). They never contain key material.
    pub fn on_event(mut self, f: impl Fn(&AcmeEvent) + Send + Sync + 'static) -> Self {
        self.sink = Some(Arc::new(f));
        self
    }

    /// Use this protocol client instead of the built-in one (tests use a fake CA here).
    pub fn client(mut self, client: impl AcmeClient) -> Self {
        self.client = Some(Box::new(client));
        self
    }

    /// How often the renewal loop looks at the certificate (default twelve hours). Renewal itself is decided by the certificate's own dates.
    pub fn check_every(mut self, every: Duration) -> Self {
        self.check_every = every;
        self
    }
}

/// What to serve and how. Built with chained calls, started with [`Site::serve`] (runs until Ctrl-C) or [`Site::start`] (returns a handle).
#[derive(Debug)]
pub struct Site {
    name: String,
    files: Option<PathBuf>,
    bind: Option<String>,
    http_bind: Option<String>,
    tls: Option<(PathBuf, PathBuf)>,
    acme: Option<Acme>,
    proxies: Vec<(String, String)>,
    adapter: Option<String>,
    dotfiles: bool,
}

impl Site {
    /// A site called `name`: the host name it is for, which is also the name its certificate is requested for.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            files: None,
            bind: None,
            http_bind: None,
            tls: None,
            acme: None,
            proxies: Vec::new(),
            adapter: None,
            dotfiles: false,
        }
    }

    /// Serve the files in this directory.
    pub fn files(mut self, dir: impl Into<PathBuf>) -> Self {
        self.files = Some(dir.into());
        self
    }

    /// Listen here (default [`DEFAULT_BIND`], or `0.0.0.0:443` with automatic certificates).
    pub fn bind(mut self, addr: impl Into<String>) -> Self {
        self.bind = Some(addr.into());
        self
    }

    /// With automatic certificates: the plain-HTTP listener that answers the HTTP-01 challenge and redirects everything else to HTTPS
    /// (default `0.0.0.0:80`).
    pub fn http_bind(mut self, addr: impl Into<String>) -> Self {
        self.http_bind = Some(addr.into());
        self
    }

    /// Serve HTTPS with this certificate chain (leaf first) and private key, both PEM files. They are read when the site starts.
    pub fn tls_pem_files(mut self, cert: impl Into<PathBuf>, key: impl Into<PathBuf>) -> Self {
        self.tls = Some((cert.into(), key.into()));
        self
    }

    /// Get and renew the certificate automatically. See [`Acme`].
    pub fn acme(mut self, acme: Acme) -> Self {
        self.acme = Some(acme);
        self
    }

    /// Forward requests under `prefix` (such as `/api`) to the app at `upstream` (such as `http://127.0.0.1:3000`), with the prefix removed:
    /// `/api/users?x=1` becomes `/users?x=1` upstream. Proxy routes are checked before the files. The forwarding headers (`X-Forwarded-For`,
    /// `-Proto`, `-Host`) are written by this server and the client's own are dropped; WebSocket upgrades are refused with `501` in this
    /// release; an upstream that is down is a `502` (`504` on a timeout) that does not say where it is.
    pub fn proxy(mut self, prefix: impl Into<String>, upstream: impl Into<String>) -> Self {
        self.proxies.push((prefix.into(), upstream.into()));
        self
    }

    /// Run on this web library instead of the default (hyper): any id HTTPAnt knows and was built with.
    pub fn adapter(mut self, id: impl Into<String>) -> Self {
        self.adapter = Some(id.into());
        self
    }

    /// Also serve files and directories whose names start with a dot (off by default: `.git`, `.env`).
    pub fn dotfiles(mut self, on: bool) -> Self {
        self.dotfiles = on;
        self
    }

    /// The name given to [`Site::new`].
    pub fn name(&self) -> &str {
        &self.name
    }

    fn server(&self) -> Result<Server> {
        Ok(match &self.adapter {
            Some(id) => Server::with_adapter(id)?,
            None => Server::new(),
        })
    }

    /// Start serving and return a handle. Everything that can be wrong with the settings is reported here, before anything listens; with
    /// automatic certificates this returns once a certificate is in service.
    pub async fn start(self) -> Result<RunningSite> {
        if self.files.is_none() && self.proxies.is_empty() {
            return Err(Error::NothingToServe);
        }
        if self.acme.is_some() && self.tls.is_some() {
            return Err(Error::TwoCertificateSources);
        }
        let plain_server = self.server()?;
        let mut server = self.server()?;
        // Proxy routes first: the router takes the first route that matches, and the file catch-all matches everything.
        for (prefix, upstream) in &self.proxies {
            let proxy = proxy::Proxy::new(prefix, upstream)?;
            let handler = {
                let proxy = proxy.clone();
                move |req: Request| {
                    let proxy = proxy.clone();
                    async move { proxy.handle(req).await }
                }
            };
            let any = [
                Method::GET,
                Method::HEAD,
                Method::POST,
                Method::PUT,
                Method::PATCH,
                Method::DELETE,
                Method::OPTIONS,
            ];
            for m in any {
                server = server
                    .route(m.clone(), proxy.prefix(), handler.clone())
                    .route(m, &format!("{}/*rest", proxy.prefix()), handler.clone());
            }
        }
        if let Some(dir) = &self.files {
            let files = files::Files::new(dir, self.dotfiles).map_err(|e| Error::FilesDir {
                path: dir.clone(),
                reason: e.to_string(),
            })?;
            let handler = move |req: Request| {
                let files = files.clone();
                async move { files.handle(req).await }
            };
            server = server.get("/", handler.clone()).get("/*rest", handler);
        }

        let Some(acme) = self.acme else {
            let tls = self.tls.is_some();
            if let Some((cert, key)) = &self.tls {
                server = server.tls(TlsAcceptor::new(FixedCert::new(load_pem(cert, key)?)));
            }
            let bind = self.bind.as_deref().unwrap_or(DEFAULT_BIND);
            let running = server.bind(bind).await?;
            return Ok(RunningSite {
                servers: vec![running],
                tls,
                acme: None,
            });
        };

        // Automatic certificates: the HTTPS listener first (its port goes into the redirect), then the plain one, then ask for the certificate.
        let mut names = vec![self.name.to_ascii_lowercase()];
        names.extend(acme.extra_names.iter().map(|n| n.to_ascii_lowercase()));
        let store_dir = match &acme.store {
            Some(d) => d.clone(),
            None => default_store().ok_or(Error::NoStore)?,
        };
        let store = acme::Store::new(store_dir);
        let challenges = Arc::new(Challenges::default());
        let client: Box<dyn AcmeClient> = match acme.client {
            Some(c) => c,
            None => Box::new(acme::InstantAcme {
                directory_url: acme.directory.clone(),
                contact: acme.contact.clone(),
                root_pem: acme.root_pem.clone(),
                store: store.clone(),
            }),
        };
        let core = acme::Core::new(acme::CoreParams {
            names: names.clone(),
            kind: acme.kind,
            directory_url: acme.directory.clone(),
            client,
            store,
            challenges: Arc::clone(&challenges),
            sink: acme.sink,
            check_every: acme.check_every,
        });
        let https = server
            .tls(TlsAcceptor::new(core.certs()))
            .bind(self.bind.as_deref().unwrap_or("0.0.0.0:443"))
            .await?;
        let http = match plain_server
            .acme_challenge(acme::HttpResponder(Arc::clone(&challenges)))
            .redirect_to_https(HttpsRedirect::new(&names).port(https.addr().port()))
            .bind(self.http_bind.as_deref().unwrap_or("0.0.0.0:80"))
            .await
        {
            Ok(h) => h,
            Err(e) => {
                let _ = https.shutdown().await;
                return Err(e.into());
            }
        };
        if let Err(e) = core.first().await {
            let _ = http.shutdown().await;
            let _ = https.shutdown().await;
            return Err(Error::Acme(e));
        }
        let renewal = tokio::spawn(Arc::clone(&core).keep_renewed());
        Ok(RunningSite {
            servers: vec![https, http],
            tls: true,
            acme: Some((core, renewal)),
        })
    }

    /// Start serving and run until Ctrl-C, then finish the requests in flight.
    pub async fn serve(self) -> Result<()> {
        let site = self.start().await?;
        let _ = tokio::signal::ctrl_c().await;
        site.shutdown().await
    }
}

fn default_store() -> Option<PathBuf> {
    let base = if cfg!(windows) {
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
    } else {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|h| Path::new(&h).join(".local").join("share"))
            })
    };
    base.map(|b| b.join("webservant"))
}

fn load_pem(cert: &Path, key: &Path) -> Result<ServerCert> {
    let read = |p: &Path| {
        std::fs::read(p).map_err(|e| Error::Certificate {
            path: Some(p.to_owned()),
            reason: e.kind().to_string(),
        })
    };
    let (chain, key_pem) = (read(cert)?, read(key)?);
    ServerCert::from_pem(&chain, &key_pem).map_err(|e| Error::Certificate {
        path: None,
        reason: e.to_string(),
    })
}

/// The certificate in service, for a site with automatic certificates.
#[derive(Clone)]
pub struct Certificates(Arc<acme::Core>);

impl Certificates {
    /// Names, the certificate's dates and issuer, and how the last attempt ended.
    pub fn status(&self) -> CertificateStatus {
        self.0.status()
    }

    /// Get a new certificate now. If it fails the current one stays in service.
    pub async fn renew_now(&self) -> Result<()> {
        self.0.renew().await.map_err(Error::Acme)
    }
}

/// A site that is listening.
pub struct RunningSite {
    servers: Vec<RunningServer>,
    tls: bool,
    acme: Option<(Arc<acme::Core>, tokio::task::JoinHandle<()>)>,
}

impl std::fmt::Debug for RunningSite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunningSite")
            .field("addr", &self.addr())
            .field("tls", &self.tls)
            .finish()
    }
}

impl RunningSite {
    /// The address actually bound for the site itself (HTTPS if there is TLS), useful with port 0.
    pub fn addr(&self) -> std::net::SocketAddr {
        self.servers[0].addr()
    }

    /// With automatic certificates, the address of the plain-HTTP listener (challenge and redirect).
    pub fn http_addr(&self) -> Option<std::net::SocketAddr> {
        self.acme
            .as_ref()
            .and_then(|_| self.servers.get(1))
            .map(RunningServer::addr)
    }

    /// `https://addr/` or `http://addr/`.
    pub fn url(&self) -> String {
        format!(
            "{}://{}/",
            if self.tls { "https" } else { "http" },
            self.addr()
        )
    }

    /// The certificate handle, when the site has automatic certificates.
    pub fn certificates(&self) -> Option<Certificates> {
        self.acme.as_ref().map(|(c, _)| Certificates(Arc::clone(c)))
    }

    /// Stop accepting and wait for requests in flight (HTTPAnt's grace period applies).
    pub async fn shutdown(self) -> Result<()> {
        if let Some((_, task)) = &self.acme {
            task.abort();
        }
        let mut first_error = None;
        for s in self.servers {
            if let Err(e) = s.shutdown().await {
                first_error.get_or_insert(e);
            }
        }
        match first_error {
            Some(e) => Err(e.into()),
            None => Ok(()),
        }
    }
}
