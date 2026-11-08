// SPDX-License-Identifier: MIT OR Apache-2.0
//! Everything around the ACME protocol, against a fake CA that implements `AcmeClient`: issuing, serving, renewal without a restart, a failed
//! renewal keeping the old certificate, the store, and refusal of names we do not hold. The real protocol is tested against Pebble in
//! `tests/acme_pebble.rs`.

use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use httpant::BoxFuture;
use httpant::Client;
use webservant::{Acme, AcmeClient, AcmeError, CertInfo, ChallengeKind, Challenges, Issued, Site};

#[derive(Clone, Default)]
struct FakeCa {
    calls: Arc<AtomicUsize>,
    fail_next: Arc<Mutex<bool>>,
    issued_pems: Arc<Mutex<Vec<String>>>,
    lifetime_days: i64,
    age_days: i64,
}

impl FakeCa {
    fn new() -> Self {
        Self {
            lifetime_days: 90,
            ..Default::default()
        }
    }
    fn latest(&self) -> String {
        self.issued_pems.lock().unwrap().last().cloned().unwrap()
    }
}

impl AcmeClient for FakeCa {
    fn obtain<'a>(
        &'a self,
        names: &'a [String],
        _c: &'a Challenges,
        _k: ChallengeKind,
    ) -> BoxFuture<'a, Result<Issued, AcmeError>> {
        Box::pin(async move {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if std::mem::take(&mut *self.fail_next.lock().unwrap()) {
                return Err(AcmeError("the fake CA is down. Try again later.".into()));
            }
            let mut params = rcgen::CertificateParams::new(names.to_vec()).unwrap();
            let now = time::OffsetDateTime::now_utc();
            params.not_before = now - time::Duration::days(self.age_days);
            params.not_after = params.not_before + time::Duration::days(self.lifetime_days);
            let key = rcgen::KeyPair::generate().unwrap();
            let cert = params.self_signed(&key).unwrap();
            self.issued_pems.lock().unwrap().push(cert.pem());
            Ok(Issued {
                chain_pem: cert.pem(),
                key_pem: key.serialize_pem(),
            })
        })
    }
}

struct Fixture {
    _tmp: tempfile::TempDir,
    root: std::path::PathBuf,
    store: std::path::PathBuf,
}

fn fixture() -> Fixture {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("public");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("index.html"), "auto hello").unwrap();
    let store = tmp.path().join("store");
    Fixture {
        _tmp: tmp,
        root,
        store,
    }
}

fn site(f: &Fixture, ca: FakeCa) -> Site {
    Site::new("LocalHost")
        .files(&f.root)
        .bind("127.0.0.1:0")
        .http_bind("127.0.0.1:0")
        .acme(
            Acme::new("me@example.test")
                .store(&f.store)
                .client(ca)
                .check_every(Duration::from_secs(3600)),
        )
}

/// Can a client that trusts only `pem` fetch the site by the name `localhost` (which is what SNI carries)?
async fn fetches(site: &webservant::RunningSite, pem: &str) -> bool {
    let c = Client::builder().add_root_pem(pem).build().unwrap();
    let url = format!("https://localhost:{}/", site.addr().port());
    match c.get(&url).send().await {
        Ok(r) => r.text().await.map(|t| t == "auto hello").unwrap_or(false),
        Err(_) => false,
    }
}

#[tokio::test]
async fn issues_serves_and_reports_status() {
    let f = fixture();
    let ca = FakeCa::new();
    let running = site(&f, ca.clone()).start().await.unwrap();
    assert_eq!(ca.calls.load(Ordering::SeqCst), 1);
    let st = running.certificates().unwrap().status();
    assert_eq!(
        st.names,
        vec!["localhost".to_string()],
        "names are lower-cased"
    );
    let info = st.cert.unwrap();
    assert!(info.not_after > info.not_before);
    assert!(matches!(st.last_result, Some(Ok(()))));
    // The store holds the private key, and the plain listener exists.
    assert!(
        fs::read_dir(f.store.join("certs"))
            .unwrap()
            .next()
            .is_some()
    );
    assert!(running.http_addr().is_some());
    // A name we do not hold is refused, not answered with this certificate: connecting by IP sends no name.
    let pem = fs::read_to_string(
        fs::read_dir(f.store.join("certs"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path()
            .join("chain.pem"),
    )
    .unwrap();
    let c = Client::builder().add_root_pem(&pem).build().unwrap();
    assert!(
        c.get(format!("https://127.0.0.1:{}/", running.addr().port()))
            .send()
            .await
            .is_err()
    );
    assert!(
        fetches(&running, &pem).await,
        "the control: the held name works"
    );
    running.shutdown().await.unwrap();
}

#[tokio::test]
async fn a_second_start_reuses_the_stored_certificate() {
    let f = fixture();
    let ca = FakeCa::new();
    site(&f, ca.clone())
        .start()
        .await
        .unwrap()
        .shutdown()
        .await
        .unwrap();
    assert_eq!(ca.calls.load(Ordering::SeqCst), 1);
    site(&f, ca.clone())
        .start()
        .await
        .unwrap()
        .shutdown()
        .await
        .unwrap();
    assert_eq!(
        ca.calls.load(Ordering::SeqCst),
        1,
        "the CA was asked again although the stored certificate was fine"
    );
    // The control: other names are another certificate, so the CA IS asked.
    Site::new("example.test")
        .files(&f.root)
        .bind("127.0.0.1:0")
        .http_bind("127.0.0.1:0")
        .acme(
            Acme::new("me@example.test")
                .store(&f.store)
                .client(ca.clone())
                .also_name("www.example.test"),
        )
        .start()
        .await
        .unwrap()
        .shutdown()
        .await
        .unwrap();
    assert_eq!(ca.calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn renewal_swaps_the_certificate_without_a_restart_and_a_failure_keeps_the_old_one() {
    let f = fixture();
    let ca = FakeCa::new();
    let running = site(&f, ca.clone()).start().await.unwrap();
    let certs = running.certificates().unwrap();
    let first = ca.latest();
    assert!(
        fetches(&running, &first).await,
        "the first certificate is not the one served"
    );

    certs.renew_now().await.unwrap();
    let second = ca.latest();
    assert_ne!(first, second);
    // The next connection gets the new certificate, on the same listener, with no restart; the old one is no longer served.
    assert!(
        fetches(&running, &second).await,
        "the renewed certificate is not served"
    );
    assert!(
        !fetches(&running, &first).await,
        "the old certificate is still served"
    );
    assert_eq!(ca.calls.load(Ordering::SeqCst), 2);

    // A failing renewal reports the failure and the certificate in service is still the second one.
    *ca.fail_next.lock().unwrap() = true;
    let err = certs.renew_now().await.unwrap_err();
    assert!(err.to_string().contains("fake CA is down"), "{err}");
    assert!(matches!(certs.status().last_result, Some(Err(_))));
    let info_after_failure = certs.status().cert.unwrap();
    let from_second = CertInfo::from_chain_pem(&second).unwrap();
    assert_eq!(info_after_failure.not_after, from_second.not_after);
    assert!(
        fetches(&running, &second).await,
        "after a failed renewal the old certificate must still be served"
    );
    running.shutdown().await.unwrap();
}

#[test]
fn due_is_read_from_the_certificates_own_dates() {
    let day = 86_400;
    let c = CertInfo {
        not_before: 0,
        not_after: 90 * day,
        issuer: "x".into(),
    };
    assert!(
        !c.due(59 * day),
        "30 days left of 90 is the edge: 59 days in is not due"
    );
    assert!(c.due(60 * day));
    // A short-lived certificate renews at a third of ITS lifetime, not at 30 days.
    let short = CertInfo {
        not_before: 0,
        not_after: 6 * day,
        issuer: "x".into(),
    };
    assert!(!short.due(3 * day) && short.due(4 * day));
    // Already expired is due; a zero-length lifetime does not divide by zero.
    assert!(c.due(100 * day));
    assert!(
        CertInfo {
            not_before: 5,
            not_after: 5,
            issuer: "x".into()
        }
        .due(5)
    );
}

#[tokio::test]
async fn acme_and_a_certificate_pair_together_are_refused() {
    let f = fixture();
    let e = site(&f, FakeCa::new())
        .tls_pem_files(f.root.join("a.pem"), f.root.join("b.pem"))
        .start()
        .await
        .unwrap_err();
    assert!(e.to_string().contains("Use one of them"), "{e}");
}

#[tokio::test]
async fn a_ca_that_is_down_at_first_start_is_an_error_naming_what_to_do() {
    let f = fixture();
    let ca = FakeCa::new();
    *ca.fail_next.lock().unwrap() = true;
    let e = site(&f, ca).start().await.unwrap_err();
    assert!(e.to_string().contains("Try again later"), "{e}");
}

/// A handshake that accepts ANY certificate, so a failure can only be the server refusing. `sni` is the name sent, if any.
async fn server_accepts(addr: std::net::SocketAddr, sni: Option<&str>) -> bool {
    use tokio_rustls::rustls::client::danger::{
        HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier,
    };
    use tokio_rustls::rustls::pki_types::{CertificateDer, ServerName, UnixTime};
    use tokio_rustls::rustls::{ClientConfig, DigitallySignedStruct, Error, SignatureScheme};

    #[derive(Debug)]
    struct AcceptAll;
    impl ServerCertVerifier for AcceptAll {
        fn verify_server_cert(
            &self,
            _: &CertificateDer<'_>,
            _: &[CertificateDer<'_>],
            _: &ServerName<'_>,
            _: &[u8],
            _: UnixTime,
        ) -> Result<ServerCertVerified, Error> {
            Ok(ServerCertVerified::assertion())
        }
        fn verify_tls12_signature(
            &self,
            _: &[u8],
            _: &CertificateDer<'_>,
            _: &DigitallySignedStruct,
        ) -> Result<HandshakeSignatureValid, Error> {
            Ok(HandshakeSignatureValid::assertion())
        }
        fn verify_tls13_signature(
            &self,
            _: &[u8],
            _: &CertificateDer<'_>,
            _: &DigitallySignedStruct,
        ) -> Result<HandshakeSignatureValid, Error> {
            Ok(HandshakeSignatureValid::assertion())
        }
        fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
            tokio_rustls::rustls::crypto::ring::default_provider()
                .signature_verification_algorithms
                .supported_schemes()
        }
    }
    let provider = Arc::new(tokio_rustls::rustls::crypto::ring::default_provider());
    let config = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .unwrap()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(AcceptAll))
        .with_no_client_auth();
    let tcp = tokio::net::TcpStream::connect(addr).await.unwrap();
    let name = match sni {
        Some(n) => ServerName::try_from(n.to_owned()).unwrap(),
        None => ServerName::IpAddress(addr.ip().into()), // an IP address: rustls sends no SNI
    };
    tokio_rustls::TlsConnector::from(Arc::new(config))
        .connect(name, tcp)
        .await
        .is_ok()
}

#[tokio::test]
async fn the_server_refuses_a_name_it_does_not_hold_and_a_client_with_no_name() {
    let f = fixture();
    let running = site(&f, FakeCa::new()).start().await.unwrap();
    let addr = running.addr();
    assert!(
        server_accepts(addr, Some("localhost")).await,
        "the control: the held name must work"
    );
    assert!(
        server_accepts(addr, Some("LOCALHOST")).await,
        "names compare without case"
    );
    assert!(
        !server_accepts(addr, Some("other.test")).await,
        "a name we do not hold got a certificate"
    );
    assert!(
        !server_accepts(addr, None).await,
        "a client with no name got a certificate"
    );
    running.shutdown().await.unwrap();
}

// ---- the contract gaps closed in slice 4 (live read-back, TLS events) ----

fn leaf_der_of(pem: &str) -> Vec<u8> {
    x509_parser::pem::Pem::iter_from_buffer(pem.as_bytes())
        .next()
        .unwrap()
        .unwrap()
        .contents
}

#[tokio::test]
async fn read_back_reports_what_the_listener_really_serves() {
    let f = fixture();
    let ca = FakeCa::new();
    let running = site(&f, ca.clone()).start().await.unwrap();
    let certs = running.certificates().unwrap();

    let served = running.served_certificate("localhost").await.unwrap();
    assert_eq!(
        served.leaf_der,
        leaf_der_of(&ca.latest()),
        "the served leaf is not the issued one"
    );
    assert_eq!(served.not_after, certs.status().cert.unwrap().not_after);

    // After a renewal the read-back shows the NEW leaf: it reads the live listener, not our records.
    let before = served.leaf_der;
    certs.renew_now().await.unwrap();
    let after = running.served_certificate("localhost").await.unwrap();
    assert_ne!(
        after.leaf_der, before,
        "the renewed certificate is not what is served"
    );
    assert_eq!(after.leaf_der, leaf_der_of(&ca.latest()));

    // A name the site does not hold is refused by the listener, and a site without TLS has nothing to read.
    assert!(running.served_certificate("other.test").await.is_err());
    running.shutdown().await.unwrap();
    let plain = Site::new("x")
        .files(&f.root)
        .bind("127.0.0.1:0")
        .start()
        .await
        .unwrap();
    assert!(
        plain
            .served_certificate("x")
            .await
            .unwrap_err()
            .to_string()
            .contains("not serving HTTPS")
    );
}

#[tokio::test]
async fn tls_events_report_refusals_and_a_renewal_taking_effect() {
    use webservant::{RefusalReason, TlsEvent};
    let f = fixture();
    let ca = FakeCa::new();
    let events: Arc<Mutex<Vec<TlsEvent>>> = Arc::default();
    let sink = Arc::clone(&events);
    let running = site(&f, ca.clone())
        .on_tls_event(move |e| sink.lock().unwrap().push(e.clone()))
        .start()
        .await
        .unwrap();
    let wait_for = |pred: &dyn Fn(&[TlsEvent]) -> bool| {
        let events = Arc::clone(&events);
        pred(&events.lock().unwrap())
    };

    assert!(!server_accepts(running.addr(), Some("other.test")).await);
    for _ in 0..40 {
        if wait_for(&|ev| {
            ev.iter().any(|e| {
                matches!(
                    e,
                    TlsEvent::HandshakeRefused {
                        reason: RefusalReason::UnknownName,
                        ..
                    }
                )
            })
        }) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(
        wait_for(&|ev| ev.iter().any(|e| matches!(
            e,
            TlsEvent::HandshakeRefused {
                reason: RefusalReason::UnknownName,
                ..
            }
        ))),
        "no refusal event: {:?}",
        events.lock().unwrap()
    );

    // The first certificate served for a name is not "changed"; the renewed one is.
    assert!(server_accepts(running.addr(), Some("localhost")).await);
    running.certificates().unwrap().renew_now().await.unwrap();
    assert!(server_accepts(running.addr(), Some("localhost")).await);
    for _ in 0..40 {
        if wait_for(&|ev| {
            ev.iter()
                .any(|e| matches!(e, TlsEvent::CertificateChanged { .. }))
        }) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let changed = events
        .lock()
        .unwrap()
        .iter()
        .filter(|e| matches!(e, TlsEvent::CertificateChanged { .. }))
        .count();
    assert_eq!(changed, 1, "{:?}", events.lock().unwrap());
    running.shutdown().await.unwrap();
}

// ---- check_reachable ----

/// The check tests count scratch directories, so they must not overlap each other (tests in one binary run in parallel).
static GATE: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn leftovers() -> usize {
    let prefix = format!("webservant-check-{}-", std::process::id());
    std::fs::read_dir(std::env::temp_dir())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.file_name().to_string_lossy().starts_with(&prefix))
        .count()
}

#[tokio::test]
async fn check_reachable_issues_once_against_staging_and_keeps_nothing() {
    let _gate = GATE.lock().await;
    let f = fixture();
    let ca = FakeCa::new();
    // `site()` uses the default directory (Let's Encrypt production) and a store inside the fixture.
    let r = site(&f, ca.clone()).check_reachable().await.unwrap();
    assert!(r.is_reachable() && r.failure.is_none());
    assert_eq!(ca.calls.load(Ordering::SeqCst), 1, "exactly one issuance");
    assert_eq!(
        r.directory,
        webservant::acme::LETS_ENCRYPT_STAGING,
        "production must be replaced by staging"
    );
    assert_eq!(r.names, vec!["localhost".to_string()]);
    assert!(
        !f.store.exists(),
        "the configured store was used or created: the check must use a throwaway one"
    );
    assert_eq!(leftovers(), 0, "the throwaway store was not removed");
}

#[tokio::test]
async fn check_reachable_keeps_a_directory_the_person_chose() {
    let _gate = GATE.lock().await;
    let f = fixture();
    let r = Site::new("localhost")
        .files(&f.root)
        .bind("127.0.0.1:0")
        .http_bind("127.0.0.1:0")
        .acme(
            Acme::new("me@example.test")
                .directory("https://ca.test/dir")
                .client(FakeCa::new()),
        )
        .check_reachable()
        .await
        .unwrap();
    assert_eq!(r.directory, "https://ca.test/dir");
}

#[tokio::test]
async fn check_reachable_reports_a_failure_as_an_answer_not_an_error() {
    let _gate = GATE.lock().await;
    let f = fixture();
    let ca = FakeCa::new();
    *ca.fail_next.lock().unwrap() = true;
    let r = site(&f, ca).check_reachable().await.unwrap();
    assert!(!r.is_reachable());
    assert!(r.failure.unwrap().contains("fake CA is down"));
    assert_eq!(leftovers(), 0);
}

#[tokio::test]
async fn check_reachable_needs_acme_and_does_not_hide_a_listener_problem() {
    let _gate = GATE.lock().await;
    let f = fixture();
    let e = Site::new("localhost")
        .files(&f.root)
        .bind("127.0.0.1:0")
        .check_reachable()
        .await
        .unwrap_err();
    assert!(
        e.to_string().contains("needs automatic certificates"),
        "{e}"
    );
    // An address in use is a problem with the setup, not an answer about reachability.
    let taken = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let e = Site::new("localhost")
        .files(&f.root)
        .bind("127.0.0.1:0")
        .http_bind(taken.local_addr().unwrap().to_string())
        .acme(Acme::new("me@example.test").client(FakeCa::new()))
        .check_reachable()
        .await
        .unwrap_err();
    assert!(e.to_string().contains("could not start"), "{e}");
    assert_eq!(leftovers(), 0);
}
