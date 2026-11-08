// SPDX-License-Identifier: MIT OR Apache-2.0
use std::fmt;
use std::path::PathBuf;

/// `Result` with [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// Why WebServAnt could not start or keep serving. Every message ends with what to do next.
#[derive(Debug)]
pub enum Error {
    /// The directory given to `files` does not exist, is not a directory, or cannot be read.
    FilesDir { path: PathBuf, reason: String },
    /// Nothing to serve: no `files` directory was given.
    NothingToServe,
    /// A certificate or key file could not be read or is not usable. Never contains key bytes.
    Certificate {
        path: Option<PathBuf>,
        reason: String,
    },
    /// Only one of the certificate and key was given.
    HalfCertificate,
    /// The server (HTTPAnt) failed: usually the address is taken or not allowed.
    Server(httpant::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FilesDir { path, reason } => write!(
                f,
                "cannot serve files from `{}`: {reason}. Give `files` an existing directory that you can read.",
                path.display()
            ),
            Self::NothingToServe => f.write_str(
                "there is nothing to serve. Call `.files(\"./public\")` (or pass `--files ./public`) with the directory to serve.",
            ),
            Self::Certificate {
                path: Some(p),
                reason,
            } => write!(
                f,
                "certificate problem with `{}`: {reason}. Check that the file exists and is PEM text (a chain, leaf first, and its private key).",
                p.display()
            ),
            Self::Certificate { path: None, reason } => write!(
                f,
                "certificate problem: {reason}. Check that the certificate chain and the key are PEM text and belong together."
            ),
            Self::HalfCertificate => f.write_str(
                "a certificate was given without its key, or a key without its certificate. Give both `tls_pem_files(cert, key)` or neither.",
            ),
            Self::Server(e) => write!(
                f,
                "the server could not start or stopped: {e}. If the address is in use, choose another with `.bind(\"127.0.0.1:8081\")`."
            ),
        }
    }
}

impl std::error::Error for Error {}

impl From<httpant::Error> for Error {
    fn from(e: httpant::Error) -> Self {
        Self::Server(e)
    }
}
