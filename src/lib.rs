// SPDX-License-Identifier: MIT OR Apache-2.0
//! # WebServAnt
//!
//! A web server that runs on the Rust web library you choose, because it is built on [HTTPAnt](https://crates.io/crates/httpant).
//! This release serves a directory of static files, over plain HTTP or over TLS from a certificate and key you already have.
//! Automatic certificates and a reverse proxy are on the roadmap (`ROADMAP.md`), not in this release.
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
//! That serves `./public` on `127.0.0.1:8080` until Ctrl-C. Nothing is exposed to the network until you ask: use `.bind("0.0.0.0:80")`
//! (or `443` with a certificate) to listen on every interface.

mod error;
mod files;

use std::path::{Path, PathBuf};

use httpant::server::{FixedCert, ServerCert, TlsAcceptor};
use httpant::{Request, RunningServer, Server};

pub use error::{Error, Result};

/// The address used when you do not choose one: this machine only.
pub const DEFAULT_BIND: &str = "127.0.0.1:8080";

/// What to serve and how. Built with chained calls, started with [`Site::serve`] (runs until Ctrl-C) or [`Site::start`] (returns a handle).
#[derive(Debug, Clone)]
pub struct Site {
    name: String,
    files: Option<PathBuf>,
    bind: String,
    tls: Option<(PathBuf, PathBuf)>,
    adapter: Option<String>,
    dotfiles: bool,
}

impl Site {
    /// A site called `name` (the host name it is for; used in messages today, and for certificates once those arrive).
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            files: None,
            bind: DEFAULT_BIND.to_owned(),
            tls: None,
            adapter: None,
            dotfiles: false,
        }
    }

    /// Serve the files in this directory.
    pub fn files(mut self, dir: impl Into<PathBuf>) -> Self {
        self.files = Some(dir.into());
        self
    }

    /// Listen here (default [`DEFAULT_BIND`]).
    pub fn bind(mut self, addr: impl Into<String>) -> Self {
        self.bind = addr.into();
        self
    }

    /// Serve HTTPS with this certificate chain (leaf first) and private key, both PEM files. They are read when the site starts.
    pub fn tls_pem_files(mut self, cert: impl Into<PathBuf>, key: impl Into<PathBuf>) -> Self {
        self.tls = Some((cert.into(), key.into()));
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

    /// Start serving and return a handle. Everything that can be wrong with the settings is reported here, before anything listens.
    pub async fn start(self) -> Result<RunningSite> {
        let dir = self.files.ok_or(Error::NothingToServe)?;
        let files = files::Files::new(&dir, self.dotfiles).map_err(|e| Error::FilesDir {
            path: dir.clone(),
            reason: e.to_string(),
        })?;
        let mut server = match &self.adapter {
            Some(id) => Server::with_adapter(id)?,
            None => Server::new(),
        };
        let handler = move |req: Request| {
            let files = files.clone();
            async move { files.handle(req).await }
        };
        server = server.get("/", handler.clone()).get("/*rest", handler);
        let tls = self.tls.is_some();
        if let Some((cert, key)) = &self.tls {
            server = server.tls(TlsAcceptor::new(FixedCert::new(load_pem(cert, key)?)));
        }
        let running = server.bind(self.bind.as_str()).await?;
        Ok(RunningSite { running, tls })
    }

    /// Start serving and run until Ctrl-C, then finish the requests in flight.
    pub async fn serve(self) -> Result<()> {
        let site = self.start().await?;
        let _ = tokio::signal::ctrl_c().await;
        site.shutdown().await
    }
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

/// A site that is listening.
#[derive(Debug)]
pub struct RunningSite {
    running: RunningServer,
    tls: bool,
}

impl RunningSite {
    /// The address actually bound (useful with port 0).
    pub fn addr(&self) -> std::net::SocketAddr {
        self.running.addr()
    }

    /// `https://addr/` or `http://addr/`.
    pub fn url(&self) -> String {
        format!(
            "{}://{}/",
            if self.tls { "https" } else { "http" },
            self.addr()
        )
    }

    /// Stop accepting and wait for requests in flight (HTTPAnt's grace period applies).
    pub async fn shutdown(self) -> Result<()> {
        Ok(self.running.shutdown().await?)
    }
}
