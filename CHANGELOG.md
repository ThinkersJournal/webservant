# Changelog

## [0.4.16] - 2026-11-27

- Docs only (README audit, adapter and `Settings` rows, run not read): a scratch crate depending on webservant `=0.4.13` from crates.io with all five adapter features, building each site from a `Settings` JSON value and `Site::from_settings(..).start()`, served a file (`200`, body matched) and a missing file (`404`) on hyper, axum, actix-web, poem, salvo and warp, Windows host; a misspelled `Settings` key is refused naming the key and the allowed ones; a site with no `files` and no proxy fails at `start()` with `there is nothing to serve`. README drops adapters and `Settings` from the not-exercised list and says SIGTERM was run on Linux in a container only (registry binary, Docker, 2026-11-26). Not exercised: ACME against a real CA, SIGTERM under systemd or on Windows, the systemd unit, file-symlink escape on Windows (creating one needs Administrator; blocked on this box, runs in Linux CI).

## [0.4.15] - 2026-11-26

- CI: portfolio workflow pinned to coderipper 467fd259 (0.48.1); `Cargo.lock` takes `url` 2.5.9 (RUSTSEC-2026-0592; `Url::origin()` is not reachable here, 0 hits in `*.rs`). No code change.
- Docs only (README audit, ACME and `check` rows, registry 0.4.13 binary against Pebble in Docker, Windows host, run not read): `webservant check` printed the documented `OK: ... Nothing was kept.` with the validation port reachable and the documented `NOT REACHABLE ... could not validate ... port 80 is reachable` with it closed (exit 1); without `--acme` it says what to add. `serve --acme` issued, served HTTPS 200 with a Pebble-issued certificate, kept `accounts/` and `certs/` (key, chain, names) under `--acme-store`, ACL on the store was the owner alone (`icacls`), a restart reused the stored certificate (Pebble logged 2 issuances in total: the check and the first serve), the HTTP side redirected a configured name with `308` to HTTPS (path and query kept) and answered `421` for another Host. Not run: a real CA (staging or production), renewal on a running server, SIGTERM (no signal sender on this Windows box), adapters, Settings, systemd.

## [0.4.14] - 2026-11-20

- Docs only (README claim-to-command audit against the registry 0.4.13, Windows, run not read): the README said a path leaving the folder is a `404`; on 0.4.13 the spellings `..`, `%2e%2e%2f`, `%5c` and a backslash answer `400` (the router refuses them before our code; `tests/files.rs` accepts either and checks no file leaks), `C:`, `::$DATA`, dotfiles and device names answer `404`. README now says so. Version number said 0.4.12; now 0.4.13 (README, CONSUMER-CONTRACT). Everything else checked held: Content-Type, ETag/304, Range 206, HEAD, `/docs` 308 to `/docs/`, `--dotfiles`, mismatched cert/key refused without key text, TLS 1.2 and 1.3 served (1.1 refused), proxy prefix strip, `X-Forwarded-*`/`Forwarded` replaced, `Connection`-named header dropped, 502 body without upstream address, WebSocket 501, PROPFIND 405, 413 over the body limit and with `--max-body`, `/` proxy with files first. Symlink escape, run after the first audit: a directory junction out of the folder answers `404` (control: a junction inside the folder is served); a file symlink could not be made here (needs Administrator), so that case was not run on this box (the test skips the same way here and runs in Linux CI). The `400` is HTTPAnt's router refusing dot segments, left as is on purpose; the README now describes what the code does. Not exercised: ACME, `check`, SIGTERM, adapters, Settings, systemd, file-symlink escape on Windows.

## [0.4.13] - 2026-11-16

- Docs only, no code change (PM notice webservant-published: 0.4.12 is on crates.io, tag v0.4.12). README, ROADMAP and CONSUMER-CONTRACT now say "on crates.io at 0.4.12" where they said first published 0.4.4 or unpublished. QUICKSTART Part 1 checked from the registry, not a checkout: `cargo install webservant --version 0.4.12` (Windows, fresh target dir, 1m19s, `Installed package webservant v0.4.12`), `webservant --version` printed `webservant 0.4.12`, `webservant serve --files ./public` printed `serving http://127.0.0.1:8080/  (Ctrl-C to stop)`, and `curl` of that address returned `200 OK` with `<h1>Hello</h1>`. Part 2 was not re-run (needs a server and a domain). Version proposed; the PM allocates it at the gate.

## [0.4.12] - 2026-11-16

- Docs only, no code change (PM task quickstart-part2-findings: httpant lane's cold-reader read of QUICKSTART Part 2, 0 blocker, 2 major, 8 minor, 3 nit; each fixed, none argued away; checked by re-reading the page and, for the proxy claim, against `tests/proxy.rs`). Major: step 1 now says steps 3 to 9 run on the server over `ssh user@SERVER-IP` and which checks are done elsewhere; `--also-name` for a `www` site is now named for steps 6, 7, 8 (a comment in the unit file) and 9. Minor: git dropped from step 1 (Part 2 never uses it), `libcap2-bin` added (for `setcap`), the build-essential parenthetical placed, rustup's "press 1" and "apt means Ubuntu/Debian" said; step 2 defines A record, DNS host vs registrar and TTL, and uses `curl -4`; step 3 says how to check `ufw` and warns about ISPs that block port 80; step 4 says where to run it and to `systemctl restart` after an upgrade; step 5 warns phone browsers rewrite `http://`; step 6 defines the authority/ACME and has a DNS-not-spread / AAAA / CAA row (worded without having seen the real message); step 7 warns the foreground server ends with the SSH session and that step 8's service orders a second real certificate; step 9 says the prefix is removed before the app sees the request and the app should bind 127.0.0.1. Nits: the top note and footer now say the same thing (an AI agent ran steps 1 to 8 on a rented server against staging); `systemctl status` and a how-to-update-the-site line in step 8; step 1 time excludes DNS and compile. Not claimed: reboot survival, rate-limit wording, other distros. Version proposed; the PM allocates it at the gate.

## [0.4.11] - 2026-11-16

- Docs and lockfile, no source change (PM notice webservant-publish-plan: the PM will publish 0.4.5 to this version in one publish, so the registry README must read right). The QUICKSTART "Rust way" heading and its intro no longer say the program is "three lines" for what is six setup commands plus a three-line `main.rs`. The failure table already gives the command-line flag first (0.4.10); it was re-read and no row quotes the Rust API to command-line users. `cargo update` (latest-versions rule): httpant-server 0.7.1 to 0.7.10 (our `~0.7.1` requirement allows it), warp 0.4.3 to 0.4.4, salvo 1.1.1 to 1.1.2 (salvo_core, salvo_macros, salvo-serde-util), rustls-webpki 0.103.16 to 0.103.17, rustls-pki-types 1.15.1 to 1.16.0, webpki-roots and webpki-root-certs 1.0.11 to 1.0.12, mio, rand, itoa, syn patch bumps, and ipnet 2.12.2 added. Version proposed; the PM allocates it at the gate.

## [0.4.10] - 2026-11-16

- Tests, CI and docs, plus two error strings (PM tasks claim-evidence, acme-pebble-loopback, letsencrypt-run-facts, and capant findings). `tests/adapters.rs` serves a file on each web library (axum, actix-web, poem, salvo, warp) and CI builds it once per `--features` value (new `adapters` job), so the README's "the web library is a setting" is now tested; with a feature off, the same test checks the error names it. The Linux CI log at e9606a2 was read: `sigterm_stops_the_server_with_a_clean_exit` ran and passed there (it is Unix-only, 0 tests on Windows). README "Errors say what to do next / every message" is softened to what is true (named problem, fix where known). `tests/acme_pebble.rs` binds its validation listener to loopback on Windows and macOS (Docker Desktop delivers `host.docker.internal` to the host's loopback), so no inbound firewall prompt per rebuilt test binary; Linux CI still binds all interfaces. Passed on Windows with Pebble in Docker. Docs: `CONSUMER-CONTRACT.md` no longer says nothing was run against Let's Encrypt (stale; staging was run 2026-11-09 to 2026-11-11), and the README, QUICKSTART and ROADMAP now say those runs were by an AI agent, not a person (PM task letsencrypt-run-facts). capant's cold-reader walk of QUICKSTART Part 1 (0 blocker, 0 major): the Rust-way heading no longer says "three lines" for six commands, "cargo home" is defined, `i/o error:` is explained, and two error messages (nothing to serve; server could not start) now give the command-line flag first and the Rust call second. The only code change is those two message strings; no behaviour change. Version proposed; the PM allocates it at the gate.

## [0.4.9] - 2026-11-16

- CI only: the shared portfolio workflow pin moves to coderipper `c72a6e03479d914de8ea8a73609ba73e93a0de92` (CodeRipper 0.47.5, PM task pin-move-0.47.5). This repo has no `.coderipper.toml` and no rule names in its files; the rule-id migration tool reports nothing to migrate. No code change. Version 0.4.9 allocated by the PM (task webservant#20-version); `docs/PATENT-SWEEP.md` now names the 0.4.x line and records that nothing after 0.2.1 was searched (WSP-013).

## [0.4.8] - 2026-11-11

- Docs only: the systemd unit (QUICKSTART step 8) run on a Debian VPS against Let's Encrypt staging (trial run 4d: start, restart with the stored certificate reused, SIGKILL then auto-restart, clean stop). QUICKSTART now says it was run (not across a reboot) and to read the log with `sudo journalctl`. No code change. Version proposed; the PM allocates it at the gate.

## [0.4.7] - 2026-11-11

- Docs only: the renewal loop on a running server, proven on Let's Encrypt staging (trial run 4c: a due certificate planted, the same process swapped it for a real staging one at the due time), `ROADMAP.md` and `HANDOFF.md` brought up to date. No code change. Version proposed; the PM allocates it at the gate.

## [0.4.6] - 2026-11-10

- HTTPAnt `~0.5.22` to `~0.7.1` (PM ruling D2, 2026-11-10: keep dependencies current). No source change was needed: `cargo build --all-targets`, `clippy -D warnings` and `cargo test` (every test target) pass on 0.7.1 on Windows; the Pebble-in-Docker tests run in Linux CI. The PM's rulings D1, D2, D5 and D8 are recorded in `docs/DESIGN.md`. Version proposed; the PM allocates it at the gate. Stacked on #16 (0.4.5).

## [0.4.5] - 2026-11-10

- `--renew-check-every SECS` on the command line (with `--acme`): how often the running server looks at its certificate, 12 hours by default; it is the `acme.check_every_secs` setting that already existed. It lets the in-process renewal loop be watched on a running server in minutes (plant a certificate that is nearly due, start, wait). The staging serve-and-renew steps A to C (serve, second start reuses, a planted due certificate is renewed at start) passed on the Debian 13 VPS and are recorded in `docs/FRESH-EYES-TRIAL.md` run 4. README marked pre-release now that 0.4.4 is published (PM: published 2026-11-10, tagged v0.4.4 at ee5a23a), and the "until the first release" install-from-source block is deleted from the quickstart. `cargo install webservant --version 0.4.4` from crates.io was run on Windows and worked (5 min compile). Version proposed; the PM allocates it at the gate.

## [0.4.4] - 2026-11-09

- Docs only: the first run of `docs/QUICKSTART.md` Part 2 against a real server and a real CA (Let's Encrypt staging, Debian 13 VPS, run by the PM's agent): `webservant check` printed the success line. Recorded as run 4 in `docs/FRESH-EYES-TRIAL.md`; three differences fixed in the quickstart (no `ufw` on some servers, harmless rustup `CFLAGS` lines, "tried on Debian 13"). `ROADMAP.md` and `HANDOFF.md` updated. No code change. Version proposed; the PM allocates it at the gate.

## [0.4.3] - 2026-11-08

- Docs only: `ROADMAP.md` rows 0, 5 and 9 brought to current state (row 0 still read "next" with D1/D2/D5/D8 open and row 9 "done in this PR", which led to a stale task), CireSnave's board 402 ruling recorded ("402: A": publish the first version on our patent sweep), and `HANDOFF.md` rewritten. No code change. Allocated by the PM.

## [0.4.2] - 2026-11-08

- `docs/QUICKSTART.md` (the five-minute hello world and the staging check as a numbered list, with what success and each failure look like). `--acme-directory URL` and `--acme-ca-root PEMFILE` on the command line (a private or test CA). A test runs `webservant check` as a real process against Pebble and asserts the two sentences the quickstart shows, and another parses every `webservant ...` command in the README and quickstart with the real argument parser. From the fresh-eyes trial (`docs/FRESH-EYES-TRIAL.md`): `--help`/`-h`/`help` and `--version` work, a mistyped option gets one line with a "did you mean", and the package ships only what a user needs (it also had a stale "placeholder" description). A malformed `bind`/`http_bind` (`8081`, `127.0.0.1:99999`) is named at start instead of being reported as "address in use". Three fresh-eyes runs are recorded. Version proposed; the PM allocates it at the gate.

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
