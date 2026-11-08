// SPDX-License-Identifier: MIT OR Apache-2.0
//! Automatic certificates (ACME, RFC 8555). The protocol is `instant-acme`'s, behind our own small [`AcmeClient`] trait so it can be replaced and
//! so tests can use a fake CA; everything around it is ours: what to ask for, answering the challenges, the store, renewal and its failures.
//!
//! HTTPAnt serves the connections. It is handed our certificate source as its `CertSource` (asked on every handshake, so a renewed certificate serves
//! the next connection) and [`Challenges`] as its HTTP-01 responder.

use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use httpant::BoxFuture;
use httpant::server::{CertSource, CertSourceError, ChallengeResponder, ServerCert};
use instant_acme::{
    Account, AccountCredentials, AuthorizationStatus, ChallengeType, Identifier, NewAccount,
    NewOrder, OrderStatus, RetryPolicy,
};

/// The Let's Encrypt production directory. Used only when the person did not choose staging or another CA.
pub const LETS_ENCRYPT_PRODUCTION: &str = "https://acme-v02.api.letsencrypt.org/directory";
/// The Let's Encrypt staging directory: certificates it issues are not trusted by browsers, and its limits are generous. The documented first run.
pub const LETS_ENCRYPT_STAGING: &str = "https://acme-staging-v02.api.letsencrypt.org/directory";

/// Which challenge to answer. Only HTTP-01 exists in this version: it needs port 80 reachable from the internet. (TLS-ALPN-01 and DNS-01 are
/// deliberately not offered; see `docs/PATENT-SWEEP.md` for what must happen before they are.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ChallengeKind {
    /// Answer on port 80 (`/.well-known/acme-challenge/<token>`).
    Http01,
}

/// A certificate request failed. The text says which step and what to do.
#[derive(Debug, Clone)]
pub struct AcmeError(pub String);

impl fmt::Display for AcmeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for AcmeError {}

/// A certificate chain and its private key, as PEM. `Debug` shows neither.
pub struct Issued {
    /// The chain, leaf first.
    pub chain_pem: String,
    /// The private key.
    pub key_pem: String,
}

impl fmt::Debug for Issued {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Issued { chain_pem: <pem>, key_pem: <redacted> }")
    }
}

/// The protocol layer: ask a certificate authority for a certificate for `names`, answering challenges through `challenges`. Implemented over
/// `instant-acme` here; a fake CA implements it in tests.
pub trait AcmeClient: Send + Sync + 'static {
    /// Obtain one certificate covering every name in `names`.
    fn obtain<'a>(
        &'a self,
        names: &'a [String],
        challenges: &'a Challenges,
        kind: ChallengeKind,
    ) -> BoxFuture<'a, Result<Issued, AcmeError>>;
}

/// The challenge answers that are pending right now. Shared between the issuer (which adds and removes) and the listeners (which read).
#[derive(Default)]
pub struct Challenges {
    http: Mutex<HashMap<String, String>>,
}

impl fmt::Debug for Challenges {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Challenges { .. }")
    }
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

impl Challenges {
    /// Answer `token` with this key authorization until [`Challenges::clear_http`].
    pub fn put_http(&self, token: &str, key_authorization: &str) {
        lock(&self.http).insert(token.to_owned(), key_authorization.to_owned());
    }
    /// Stop answering `token`.
    pub fn clear_http(&self, token: &str) {
        lock(&self.http).remove(token);
    }
}

/// HTTPAnt asks this for an HTTP-01 token.
#[derive(Clone, Debug)]
pub(crate) struct HttpResponder(pub(crate) Arc<Challenges>);

impl ChallengeResponder for HttpResponder {
    fn key_authorization<'a>(&'a self, token: &'a str) -> BoxFuture<'a, Option<String>> {
        let found = lock(&self.0.http).get(token).cloned();
        Box::pin(async move { found })
    }
}

/// HTTPAnt's certificate source: the issued certificate for the names we hold.
/// A name we do not hold, and a client that sends no name, are refused: never another site's certificate.
#[derive(Clone)]
pub(crate) struct AcmeCerts {
    served: Arc<RwLock<HashMap<String, Arc<ServerCert>>>>,
}

impl AcmeCerts {
    fn new() -> Self {
        Self {
            served: Arc::default(),
        }
    }
    fn install(&self, names: &[String], cert: ServerCert) {
        let cert = Arc::new(cert);
        let mut map = self.served.write().unwrap_or_else(|e| e.into_inner());
        map.clear();
        for n in names {
            map.insert(n.to_ascii_lowercase(), Arc::clone(&cert));
        }
    }
}

impl CertSource for AcmeCerts {
    fn cert_for<'a>(
        &'a self,
        server_name: Option<&'a str>,
    ) -> BoxFuture<'a, Result<Option<Arc<ServerCert>>, CertSourceError>> {
        let found = server_name.and_then(|n| {
            self.served
                .read()
                .unwrap_or_else(|e| e.into_inner())
                .get(&n.to_ascii_lowercase())
                .cloned()
        });
        Box::pin(async move { Ok(found) })
    }
}

/// What a certificate says about itself.
#[derive(Debug, Clone)]
pub struct CertInfo {
    /// Start of validity, seconds since the epoch.
    pub not_before: i64,
    /// End of validity, seconds since the epoch.
    pub not_after: i64,
    /// The issuer's common name (or its full name if it has none).
    pub issuer: String,
}

impl CertInfo {
    /// Read the first certificate in a PEM chain.
    pub fn from_chain_pem(chain_pem: &str) -> Result<Self, AcmeError> {
        let pem = x509_parser::pem::Pem::iter_from_buffer(chain_pem.as_bytes())
            .next()
            .ok_or_else(|| AcmeError("the certificate chain holds no certificate".into()))?
            .map_err(|e| AcmeError(format!("the certificate chain is not valid PEM: {e}")))?;
        let (_, cert) = x509_parser::parse_x509_certificate(&pem.contents)
            .map_err(|e| AcmeError(format!("the certificate cannot be read: {e}")))?;
        let issuer = cert
            .issuer()
            .iter_common_name()
            .next()
            .and_then(|cn| cn.as_str().ok())
            .map_or_else(|| cert.issuer().to_string(), str::to_owned);
        Ok(Self {
            not_before: cert.validity().not_before.timestamp(),
            not_after: cert.validity().not_after.timestamp(),
            issuer,
        })
    }

    /// Renewal is due when a third of the lifetime or less is left (30 days of a 90-day certificate). Read from the certificate, never a constant,
    /// so shorter lifetimes work without a change.
    pub fn due(&self, now: i64) -> bool {
        let lifetime = (self.not_after - self.not_before).max(0);
        now >= self.not_after - lifetime / 3
    }

    /// The end of validity as a `SystemTime`.
    pub fn not_after_time(&self) -> SystemTime {
        if self.not_after >= 0 {
            UNIX_EPOCH + Duration::from_secs(self.not_after as u64)
        } else {
            UNIX_EPOCH
        }
    }
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

// ---------------------------------------------------------------------------------------------------------------------------------------
// The store
// ---------------------------------------------------------------------------------------------------------------------------------------

/// Where the account and the certificate live. Written atomically, mode 0600 on Unix. (On Windows the default location is under the user's own
/// profile, which only that user and administrators can read; tightening the ACL explicitly is a roadmap row.)
#[derive(Debug, Clone)]
pub(crate) struct Store {
    dir: PathBuf,
}

impl Store {
    pub(crate) fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    fn account_file(&self, directory_url: &str) -> PathBuf {
        // Staging and production accounts are different accounts, so each directory gets its own file.
        let tag: String = directory_url
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect();
        self.dir.join("accounts").join(format!("{tag}.json"))
    }

    fn cert_dir(&self, directory_url: &str) -> PathBuf {
        let tag: String = directory_url
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect();
        self.dir.join("certs").join(tag)
    }

    pub(crate) fn load_account(&self, url: &str) -> Result<Option<AccountCredentials>, AcmeError> {
        match std::fs::read(self.account_file(url)) {
            Ok(b) => serde_json::from_slice(&b)
                .map(Some)
                .map_err(|e| AcmeError(format!("the stored account file is damaged ({e}). Delete `{}` to create a new account.", self.account_file(url).display()))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(AcmeError(format!("cannot read the stored account: {e}. Check the permissions of `{}`.", self.dir.display()))),
        }
    }

    pub(crate) fn save_account(
        &self,
        url: &str,
        creds: &AccountCredentials,
    ) -> Result<(), AcmeError> {
        let bytes = serde_json::to_vec(creds).map_err(|e| AcmeError(e.to_string()))?;
        write_private(&self.account_file(url), &bytes)
    }

    /// The stored certificate for exactly these names, if there is one.
    pub(crate) fn load_cert(&self, url: &str, names: &[String]) -> Option<Issued> {
        let dir = self.cert_dir(url);
        let stored_names = std::fs::read_to_string(dir.join("names.txt")).ok()?;
        if stored_names.lines().map(str::trim).collect::<Vec<_>>()
            != names.iter().map(String::as_str).collect::<Vec<_>>()
        {
            return None;
        }
        Some(Issued {
            chain_pem: std::fs::read_to_string(dir.join("chain.pem")).ok()?,
            key_pem: std::fs::read_to_string(dir.join("key.pem")).ok()?,
        })
    }

    pub(crate) fn save_cert(
        &self,
        url: &str,
        names: &[String],
        issued: &Issued,
    ) -> Result<(), AcmeError> {
        let dir = self.cert_dir(url);
        write_private(&dir.join("key.pem"), issued.key_pem.as_bytes())?;
        write_private(&dir.join("chain.pem"), issued.chain_pem.as_bytes())?;
        // Written last: a crash before this leaves a certificate that is not used, never one for the wrong names.
        write_private(&dir.join("names.txt"), names.join("\n").as_bytes())
    }
}

fn write_private(path: &Path, bytes: &[u8]) -> Result<(), AcmeError> {
    let fail = |e: std::io::Error| {
        AcmeError(format!(
            "cannot write `{}`: {e}. Check that the store directory exists and is writable.",
            path.display()
        ))
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(fail)?;
    }
    let tmp = path.with_extension("tmp");
    {
        use std::io::Write;
        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        let mut f = opts.open(&tmp).map_err(fail)?;
        f.write_all(bytes).map_err(fail)?;
        f.sync_all().map_err(fail)?;
    }
    std::fs::rename(&tmp, path).map_err(fail)
}

// ---------------------------------------------------------------------------------------------------------------------------------------
// instant-acme
// ---------------------------------------------------------------------------------------------------------------------------------------

/// [`AcmeClient`] over `instant-acme`.
pub(crate) struct InstantAcme {
    pub(crate) directory_url: String,
    pub(crate) contact: Option<String>,
    pub(crate) root_pem: Option<PathBuf>,
    pub(crate) store: Store,
}

fn api<E: fmt::Display>(step: &str) -> impl FnOnce(E) -> AcmeError + '_ {
    move |e| {
        AcmeError(format!(
            "the certificate authority refused or failed at `{step}`: {e}"
        ))
    }
}

impl InstantAcme {
    async fn account(&self) -> Result<Account, AcmeError> {
        let builder = || match &self.root_pem {
            Some(p) => Account::builder_with_root(p),
            None => Account::builder(),
        };
        if let Some(creds) = self.store.load_account(&self.directory_url)? {
            return builder()
                .map_err(api("start"))?
                .from_credentials(creds)
                .await
                .map_err(api("load account"));
        }
        let mailto = self.contact.as_ref().map(|c| format!("mailto:{c}"));
        let contacts: Vec<&str> = mailto.iter().map(String::as_str).collect();
        let (account, creds) = builder()
            .map_err(api("start"))?
            .create(
                &NewAccount {
                    contact: &contacts,
                    terms_of_service_agreed: true,
                    only_return_existing: false,
                },
                self.directory_url.clone(),
                None,
            )
            .await
            .map_err(api("create account"))?;
        self.store.save_account(&self.directory_url, &creds)?;
        Ok(account)
    }

    async fn run(
        &self,
        names: &[String],
        challenges: &Challenges,
        kind: ChallengeKind,
        cleanup: &mut Vec<Cleanup>,
    ) -> Result<Issued, AcmeError> {
        let account = self.account().await?;
        let ids: Vec<Identifier> = names.iter().map(|n| Identifier::Dns(n.clone())).collect();
        let mut order = account
            .new_order(&NewOrder::new(&ids))
            .await
            .map_err(api("new order"))?;
        let ChallengeKind::Http01 = kind;
        {
            let mut auths = order.authorizations();
            while let Some(next) = auths.next().await {
                let mut authz = next.map_err(api("authorizations"))?;
                match authz.status {
                    AuthorizationStatus::Pending => {}
                    AuthorizationStatus::Valid => continue,
                    other => {
                        return Err(AcmeError(format!(
                            "a name is in state {other:?} at the certificate authority. Wait a while and start again; if it repeats, check the name is spelled right and its DNS points at this machine."
                        )));
                    }
                }
                let mut chal = authz.challenge(ChallengeType::Http01).ok_or_else(|| {
                    AcmeError("the certificate authority did not offer HTTP-01 for this name. Check that the name is a plain host name (wildcards need DNS, which is not supported).".to_owned())
                })?;
                let ka = chal.key_authorization();
                let token = chal.token.clone();
                challenges.put_http(&token, ka.as_str());
                cleanup.push(Cleanup::Http(token));
                chal.set_ready().await.map_err(api("challenge ready"))?;
            }
        }
        let status = order
            .poll_ready(&RetryPolicy::default().timeout(Duration::from_secs(90)))
            .await
            .map_err(|e| self.validation_failed(e))?;
        if status != OrderStatus::Ready {
            return Err(self.validation_failed(format!("order is {status:?}")));
        }
        let key_pem = order.finalize().await.map_err(api("finalize"))?;
        let chain_pem = order
            .poll_certificate(&RetryPolicy::default().timeout(Duration::from_secs(60)))
            .await
            .map_err(api("download certificate"))?;
        Ok(Issued { chain_pem, key_pem })
    }

    fn validation_failed(&self, why: impl fmt::Display) -> AcmeError {
        AcmeError(format!(
            "the certificate authority could not validate the name: {why}. Check that the name's DNS points at this machine and that port 80 is reachable from the internet (HTTP-01 needs it). Try `--staging` first to avoid rate limits while you fix it."
        ))
    }
}

enum Cleanup {
    Http(String),
}

impl AcmeClient for InstantAcme {
    fn obtain<'a>(
        &'a self,
        names: &'a [String],
        challenges: &'a Challenges,
        kind: ChallengeKind,
    ) -> BoxFuture<'a, Result<Issued, AcmeError>> {
        Box::pin(async move {
            let mut cleanup = Vec::new();
            let result = self.run(names, challenges, kind, &mut cleanup).await;
            for c in cleanup {
                match c {
                    Cleanup::Http(t) => challenges.clear_http(&t),
                }
            }
            result
        })
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------------
// The manager: first issuance, renewal, status, events
// ---------------------------------------------------------------------------------------------------------------------------------------

/// What happened, for an audit trail. No key material, ever.
#[derive(Debug, Clone)]
pub enum AcmeEvent {
    /// A stored certificate was loaded at start.
    Loaded { names: Vec<String>, not_after: i64 },
    /// A new certificate was obtained and is being served.
    Issued {
        names: Vec<String>,
        not_after: i64,
        issuer: String,
    },
    /// A renewal failed; the current certificate stays in service and the next try is in `retry_in`.
    RenewalFailed { reason: String, retry_in: Duration },
}

/// Where [`AcmeEvent`]s go.
pub type EventSink = Arc<dyn Fn(&AcmeEvent) + Send + Sync>;
type Sink = EventSink;

/// A snapshot of the certificate in service.
#[derive(Debug, Clone)]
pub struct CertificateStatus {
    /// The names the certificate covers.
    pub names: Vec<String>,
    /// The certificate in service, if any yet.
    pub cert: Option<CertInfo>,
    /// How the last attempt ended (`None` before any attempt).
    pub last_result: Option<Result<(), String>>,
}

pub(crate) struct CoreParams {
    pub(crate) names: Vec<String>,
    pub(crate) kind: ChallengeKind,
    pub(crate) directory_url: String,
    pub(crate) client: Box<dyn AcmeClient>,
    pub(crate) store: Store,
    pub(crate) challenges: Arc<Challenges>,
    pub(crate) sink: Option<Sink>,
    pub(crate) check_every: Duration,
}

pub(crate) struct Core {
    pub(crate) names: Vec<String>,
    pub(crate) kind: ChallengeKind,
    pub(crate) directory_url: String,
    pub(crate) client: Box<dyn AcmeClient>,
    pub(crate) store: Store,
    pub(crate) challenges: Arc<Challenges>,
    pub(crate) certs: AcmeCerts,
    pub(crate) sink: Option<Sink>,
    pub(crate) check_every: Duration,
    state: Mutex<CertificateStatus>,
}

impl Core {
    pub(crate) fn new(p: CoreParams) -> Arc<Self> {
        let CoreParams {
            names,
            kind,
            directory_url,
            client,
            store,
            challenges,
            sink,
            check_every,
        } = p;
        let certs = AcmeCerts::new();
        let state = Mutex::new(CertificateStatus {
            names: names.clone(),
            cert: None,
            last_result: None,
        });
        Arc::new(Self {
            names,
            kind,
            directory_url,
            client,
            store,
            challenges,
            certs,
            sink,
            check_every,
            state,
        })
    }

    pub(crate) fn certs(&self) -> AcmeCerts {
        self.certs.clone()
    }

    fn emit(&self, e: AcmeEvent) {
        if let Some(s) = &self.sink {
            s(&e);
        }
    }

    pub(crate) fn status(&self) -> CertificateStatus {
        lock(&self.state).clone()
    }

    fn put_in_service(&self, issued: &Issued) -> Result<CertInfo, AcmeError> {
        let info = CertInfo::from_chain_pem(&issued.chain_pem)?;
        let cert = ServerCert::from_pem(issued.chain_pem.as_bytes(), issued.key_pem.as_bytes())
            .map_err(|e| AcmeError(format!("the certificate we were given is unusable: {e}")))?;
        self.certs.install(&self.names, cert);
        lock(&self.state).cert = Some(info.clone());
        Ok(info)
    }

    /// At start: use the stored certificate if it is still good, else obtain one. Returns when a certificate is in service.
    pub(crate) async fn first(&self) -> Result<(), AcmeError> {
        if let Some(stored) = self.store.load_cert(&self.directory_url, &self.names)
            && let Ok(info) = self.put_in_service(&stored)
        {
            self.emit(AcmeEvent::Loaded {
                names: self.names.clone(),
                not_after: info.not_after,
            });
            if !info.due(now_secs()) {
                return Ok(());
            }
            // Due or past due: keep serving it and try to renew; if that fails and it has not expired, carry on with it.
            return match self.renew().await {
                Ok(()) => Ok(()),
                Err(e) if info.not_after > now_secs() => {
                    self.emit(AcmeEvent::RenewalFailed {
                        reason: e.to_string(),
                        retry_in: self.check_every,
                    });
                    Ok(())
                }
                Err(e) => Err(e),
            };
        }
        self.renew().await
    }

    /// Obtain a certificate now. On failure the certificate in service (if any) stays in service.
    pub(crate) async fn renew(&self) -> Result<(), AcmeError> {
        let result: Result<CertInfo, AcmeError> = async {
            let issued = self
                .client
                .obtain(&self.names, &self.challenges, self.kind)
                .await?;
            let info = self.put_in_service(&issued)?;
            self.store
                .save_cert(&self.directory_url, &self.names, &issued)?;
            Ok(info)
        }
        .await;
        match result {
            Ok(info) => {
                lock(&self.state).last_result = Some(Ok(()));
                self.emit(AcmeEvent::Issued {
                    names: self.names.clone(),
                    not_after: info.not_after,
                    issuer: info.issuer,
                });
                Ok(())
            }
            Err(e) => {
                lock(&self.state).last_result = Some(Err(e.to_string()));
                Err(e)
            }
        }
    }

    /// The renewal loop: wake every `check_every`; renew when due; on failure back off (a minute, doubling to an hour) and never stop.
    pub(crate) async fn keep_renewed(self: Arc<Self>) {
        let mut wait = self.check_every;
        loop {
            tokio::time::sleep(wait).await;
            let due = lock(&self.state)
                .cert
                .as_ref()
                .is_none_or(|c| c.due(now_secs()));
            if !due {
                wait = self.check_every;
                continue;
            }
            match self.renew().await {
                Ok(()) => wait = self.check_every,
                Err(e) => {
                    let current = if wait == self.check_every {
                        Duration::from_secs(60).min(self.check_every)
                    } else {
                        (wait * 2)
                            .min(Duration::from_secs(3600))
                            .max(Duration::from_secs(1))
                    };
                    wait = current;
                    self.emit(AcmeEvent::RenewalFailed {
                        reason: e.to_string(),
                        retry_in: wait,
                    });
                }
            }
        }
    }
}
