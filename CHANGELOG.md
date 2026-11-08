# Changelog

## [0.4.2] - 2026-11-08

- `docs/QUICKSTART.md` (the five-minute hello world and the staging check as a numbered list, with what success and each failure look like). `--acme-directory URL` and `--acme-ca-root PEMFILE` on the command line (a private or test CA). A test runs `webservant check` as a real process against Pebble and asserts the two sentences the quickstart shows, and another parses every `webservant ...` command in the README and quickstart with the real argument parser. Version proposed; the PM allocates it at the gate.

## [0.4.1] - 2026-11-08

- A proxy at `/` (`.proxy("/", url)`, `--proxy /=url`): the app answers everything else, with files that exist served first (GET/HEAD only; a directory needs an `index.html`; a dotfile or a missing file goes to the app). One root proxy only; prefix proxies still win. Version proposed; the PM allocates it at the gate.

## [0.4.0] - 2026-11-08

- Breaking (pre-1.0, public structs gained fields): `Settings::max_body_bytes`, `ProxyRoute::{connect,response,idle}_timeout_secs`. New: `Site::max_body_bytes`, `Site::proxy_with` and `ProxyOptions` (per-route timeouts; zero is refused), `--max-body BYTES`, and the matching keys in `configurant.toml`. Version proposed; the PM allocates it at the gate.

## [0.3.4] - 2026-11-08

- The ACME store directory is made private before the first write: mode 0700 on Unix; on Windows inherited permissions are removed and only the current user is granted access (`icacls`, by SID, system tools by full path). If it cannot be made private nothing is written. Version proposed; the PM allocates it at the gate.

## [0.3.3] - 2026-11-08

- `configurant.toml`: WebServAnt's settings as a ConfigurAnt schema, tested with ConfigurAnt 0.6.3 (dev-dependency). An `acme` or `tls` table with nothing set now counts as absent (ConfigurAnt supplies empty ones). Version proposed; the PM allocates it at the gate.

## [0.3.2] - 2026-11-08

- `Site::check_reachable()` and `Reachability`; `webservant check`: one real issuance with a throwaway store (production replaced by staging), reporting which step failed. Version proposed; the PM allocates it at the gate.

## [0.3.1] - 2026-11-08

- `Settings`, `TlsFiles`, `AcmeSettings`, `ProxyRoute` and `Site::from_settings`: the whole configuration as serde data (unknown keys refused). The command line now goes through it. Version proposed; the PM allocates it at the gate.

## [0.3.0] - 2026-11-08

- Breaking (pre-1.0): the unreachable TLS-ALPN-01 code is removed (`ChallengeKind` is `#[non_exhaustive]` with only `Http01`; `Challenges::put_alpn`/`clear_alpn` are gone) so the first version has none of the four mechanisms that need a re-sweep (on-demand issuance, DNS-01, TLS-ALPN-01, WebSocket). `rcgen` is no longer a dependency (tests only).
- `RunningSite::served_certificate(name)`: read back what the HTTPS listener really serves. `Site::on_tls_event`: HTTPAnt's refused-handshake and certificate-changed events. `webservant::shutdown_signal()` and SIGTERM / console-close handling in `serve` and the binary. Version proposed; the PM allocates it at the gate.

## [0.2.2] - 2026-11-08

- Docs only: `docs/CONSUMER-CONTRACT.md` (what WebHostAnt's local serving gets, and what it does not). Version proposed.

## [0.2.1] - 2026-11-08

- Reverse proxy: `Site::proxy`, `webservant serve --proxy PREFIX=UPSTREAM`. Enables HTTPAnt's `https` client feature (https upstreams). Version proposed; the PM allocates it at the gate.

## [0.2.0] - 2026-11-08

- Automatic certificates: `Acme`, `Site::acme`, `webservant serve --acme`; HTTP-01; store; renewal; events; `RunningSite::certificates()`. Breaking (pre-1.0): `Site` is no longer `Clone`; `RunningSite` is no longer `Debug`-derived (hand-written). Version proposed; the PM allocates it at the gate.

## [0.1.0] - 2026-11-08

- First functionality: `Site` (static files, `index.html`, `ETag`/`304`, `Range`, `HEAD`, dotfiles off by default, traversal and symlink-escape refused), HTTPS from a PEM certificate and key, the `webservant serve` command, the web library as a setting. No ACME, no proxy yet. Version number proposed; the PM allocates it at the gate.

## [0.0.1] - 2026-11-08

- Docs only: design draft (`docs/DESIGN.md`) and roadmap rows. Still no functionality.

## [0.0.0] - 2026-10-28

- Name reserved with an empty placeholder crate.
