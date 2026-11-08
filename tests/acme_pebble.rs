// SPDX-License-Identifier: MIT OR Apache-2.0
//! The real ACME protocol against Pebble (Let's Encrypt's own test CA), in Docker, for HTTP-01. (TLS-ALPN-01 is not offered yet: HTTPAnt's `ServerCert` rejects the RFC 8737 validation certificate, whose `acmeIdentifier` extension is critical. See ROADMAP.) Pebble validates by connecting back
//! to this machine as `host.docker.internal`, on its validation port 5002.
//!
//! No Docker means the test cannot run: it says SKIPPED, unless `WEBSERVANT_REQUIRE_PEBBLE` is set, as it is in CI, where that is a failure.

use std::fs;
use std::process::Command;
use std::time::Duration;

use httpant::Client;
use webservant::{Acme, Site};

const NAME: &str = "host.docker.internal";
const CONTAINER: &str = "webservant-pebble-test";

struct Pebble {
    minica: String,
    minica_path: std::path::PathBuf,
    _dir: tempfile::TempDir,
}

impl Drop for Pebble {
    fn drop(&mut self) {
        let _ = Command::new("docker")
            .args(["rm", "-f", CONTAINER])
            .output();
    }
}

fn docker(args: &[&str]) -> std::io::Result<std::process::Output> {
    Command::new("docker").args(args).output()
}

async fn start_pebble() -> Option<Pebble> {
    let required = std::env::var_os("WEBSERVANT_REQUIRE_PEBBLE").is_some();
    // Pebble is a Linux image: a daemon that only runs Windows containers (the Windows CI runner) cannot run it.
    let up = docker(&["info", "--format", "{{.OSType}}"])
        .is_ok_and(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).trim() == "linux");
    if !up {
        assert!(
            !required,
            "WEBSERVANT_REQUIRE_PEBBLE is set but Docker is not usable"
        );
        eprintln!("SKIPPED: Docker is not available, so the Pebble tests did not run");
        return None;
    }
    let _ = docker(&["rm", "-f", CONTAINER]);
    let run = docker(&[
        "run",
        "-d",
        "--name",
        CONTAINER,
        "-p",
        "14000:14000",
        "-p",
        "15000:15000",
        "-e",
        "PEBBLE_VA_NOSLEEP=1",
        "-e",
        "PEBBLE_WFE_NONCEREJECT=0",
        "-e",
        "PEBBLE_AUTHZREUSE=0",
        "--add-host=host.docker.internal:host-gateway",
        "ghcr.io/letsencrypt/pebble:latest",
    ])
    .expect("docker run");
    assert!(
        run.status.success(),
        "docker run failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    let dir = tempfile::tempdir().unwrap();
    let minica_path = dir.path().join("minica.pem");
    let cp = docker(&[
        "cp",
        &format!("{CONTAINER}:/test/certs/pebble.minica.pem"),
        minica_path.to_str().unwrap(),
    ])
    .unwrap();
    assert!(
        cp.status.success(),
        "docker cp failed: {}",
        String::from_utf8_lossy(&cp.stderr)
    );
    let minica = fs::read_to_string(&minica_path).unwrap();
    let c = Client::builder().add_root_pem(&minica).build().unwrap();
    for _ in 0..60 {
        if c.get("https://localhost:14000/dir").send().await.is_ok() {
            return Some(Pebble {
                minica,
                minica_path,
                _dir: dir,
            });
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    panic!("Pebble did not answer on https://localhost:14000/dir within 30 s");
}

async fn issue_and_serve(p: &Pebble) {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("public");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("index.html"), "from pebble").unwrap();
    let acme = Acme::new("test@example.com")
        .directory("https://localhost:14000/dir")
        .ca_root_pem_file(&p.minica_path)
        .store(tmp.path().join("store"));
    // The listener Pebble connects to must be on its validation port 5002; the HTTPS one is free.
    let (https_bind, http_bind) = ("127.0.0.1:0", "0.0.0.0:5002");
    let site = Site::new(NAME)
        .files(&root)
        .bind(https_bind)
        .http_bind(http_bind)
        .acme(acme)
        .start()
        .await
        .unwrap_or_else(|e| panic!("start: {e}"));
    let st = site.certificates().unwrap().status();
    let info = st.cert.expect("a certificate in service");
    assert!(
        info.issuer.contains("Pebble Intermediate CA"),
        "issuer was {}",
        info.issuer
    );

    // Trust only Pebble's real issuance root (from its management API) and do a TLS handshake with the site, sending SNI `NAME`: the
    // certificate chains to that root and names the host, so the whole path (order, validation, finalize, download, serving) worked. Done on
    // loopback with a raw handshake, so it needs neither DNS for `NAME` nor a firewall opening on the machine's LAN address.
    let mgmt = Client::builder().add_root_pem(&p.minica).build().unwrap();
    let root_pem = mgmt
        .get("https://localhost:15000/roots/0")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let port = site.addr().port();
    assert!(
        handshake(&root_pem, NAME, port).await.is_ok(),
        "handshake with the Pebble root"
    );
    // The controls: the wrong trust root fails (so the check does verify the chain), and so does another name (so it verifies the name).
    assert!(
        handshake(&p.minica, NAME, port).await.is_err(),
        "minica must not verify the issued chain"
    );
    assert!(
        handshake(&root_pem, "other.example", port).await.is_err(),
        "another name must not verify"
    );
    site.shutdown().await.unwrap();
}

#[tokio::test]
async fn pebble_issues_for_http01() {
    let Some(p) = start_pebble().await else {
        return;
    };
    issue_and_serve(&p).await;
    check_reachable_against_pebble(&p).await;
    cli_check_against_pebble(&p);
}

/// The `webservant check` command line, run as a real process against Pebble. The two sentences docs/QUICKSTART.md shows a person (what success
/// looks like, what failure looks like) are the ones asserted here, so the page cannot drift from the program.
fn cli_check_against_pebble(p: &Pebble) {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("public");
    fs::create_dir_all(&root).unwrap();
    let run = |http_bind: &str| {
        Command::new(env!("CARGO_BIN_EXE_webservant"))
            .args([
                "check",
                "--name",
                NAME,
                "--acme",
                "test@example.com",
                "--acme-directory",
                "https://localhost:14000/dir",
                "--acme-ca-root",
            ])
            .arg(&p.minica_path)
            .arg("--files")
            .arg(&root)
            .args(["--bind", "127.0.0.1:0", "--http-bind", http_bind])
            .output()
            .unwrap()
    };
    let ok = run("0.0.0.0:5002");
    let out = String::from_utf8_lossy(&ok.stdout);
    assert!(
        ok.status.success(),
        "{out} {}",
        String::from_utf8_lossy(&ok.stderr)
    );
    assert_eq!(
        out.trim(),
        format!(
            "OK: https://localhost:14000/dir issued a certificate for {NAME}. Nothing was kept."
        )
    );

    let bad = run("127.0.0.1:0");
    let err = String::from_utf8_lossy(&bad.stderr);
    assert_eq!(bad.status.code(), Some(1), "{err}");
    assert!(
        err.starts_with("NOT REACHABLE (https://localhost:14000/dir): ")
            && err.contains("could not validate")
            && err.contains("port 80 is reachable from the internet"),
        "{err}"
    );
}

/// `check_reachable` against the real protocol: a reachable setup is reported reachable, and one whose validation port Pebble cannot reach
/// is reported NOT reachable with a sentence about the validation (the answer the check exists to give).
async fn check_reachable_against_pebble(p: &Pebble) {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("public");
    fs::create_dir_all(&root).unwrap();
    let site = |http_bind: &str| {
        Site::new(NAME)
            .files(&root)
            .bind("127.0.0.1:0")
            .http_bind(http_bind)
            .acme(
                Acme::new("test@example.com")
                    .directory("https://localhost:14000/dir")
                    .ca_root_pem_file(&p.minica_path),
            )
    };
    let ok = site("0.0.0.0:5002").check_reachable().await.unwrap();
    assert!(
        ok.is_reachable(),
        "reachable setup reported as failing: {:?}",
        ok.failure
    );
    assert_eq!(ok.directory, "https://localhost:14000/dir");

    // Nothing listens where Pebble looks (port 5002), so validation must fail, and the report must say so.
    let bad = site("127.0.0.1:0").check_reachable().await.unwrap();
    assert!(
        !bad.is_reachable(),
        "an unreachable setup was reported reachable"
    );
    let why = bad.failure.unwrap();
    assert!(
        why.contains("could not validate") && why.contains("port 80"),
        "{why}"
    );
}

/// A TLS handshake to 127.0.0.1:`port` with SNI `name`, verifying the server's chain against the roots in `root_pem` and its name.
async fn handshake(root_pem: &str, name: &str, port: u16) -> Result<(), String> {
    use std::sync::Arc;
    use tokio_rustls::rustls::pki_types::pem::PemObject;
    use tokio_rustls::rustls::pki_types::{CertificateDer, ServerName};
    use tokio_rustls::rustls::{ClientConfig, RootCertStore};
    let mut roots = RootCertStore::empty();
    for cert in CertificateDer::pem_slice_iter(root_pem.as_bytes()) {
        roots
            .add(cert.map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    }
    let provider = Arc::new(tokio_rustls::rustls::crypto::ring::default_provider());
    let config = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|e| e.to_string())?
        .with_root_certificates(roots)
        .with_no_client_auth();
    let tcp = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .map_err(|e| e.to_string())?;
    let server_name = ServerName::try_from(name.to_owned()).map_err(|e| e.to_string())?;
    tokio_rustls::TlsConnector::from(Arc::new(config))
        .connect(server_name, tcp)
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}
