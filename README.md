# WebServAnt

**Serve a folder of files as a website, on the Rust web library you choose.**

> **Pre-release (0.x).** The first published version is 0.4.4. Until 1.0 a breaking change bumps the second number (0.4 to 0.5), so pin it (`webservant = "0.4"`) and read the [changelog](https://github.com/ThinkersJournal/webservant/blob/main/CHANGELOG.md) before upgrading. What has not been run yet is listed in [docs/FRESH-EYES-TRIAL.md](https://github.com/ThinkersJournal/webservant/blob/main/docs/FRESH-EYES-TRIAL.md).

```rust
#[tokio::main]
async fn main() -> webservant::Result<()> {
    webservant::Site::new("example.org").files("./public").serve().await
}
```

**New here? Follow [docs/QUICKSTART.md](https://github.com/ThinkersJournal/webservant/blob/main/docs/QUICKSTART.md)**: a page on your screen in five minutes, then a real `https://` site, tested first.

The program above serves `./public` at `http://127.0.0.1:8080/` until Ctrl-C. It listens on this machine only, so nothing is exposed until you say so.
The same thing without writing code:

```sh
cargo install webservant
mkdir public && echo "<h1>Hello</h1>" > public/index.html
webservant serve --files ./public
```

The Rust version prints nothing while it runs (that is success); the command prints one `serving ...` line. (Before 0.4.4, the first release, `cargo install webservant` could not find it.)

To use it as a library: `cargo add webservant` and `cargo add tokio --features macros,rt-multi-thread`. The repository's CI also runs
[`examples/hello.rs`](https://github.com/ThinkersJournal/webservant/blob/main/examples/hello.rs), a version that serves a file, fetches it, checks it
and stops by itself, so the example cannot go stale.

## What this release does

The list below is the detail; you do not need any of it to get started (the quickstart above is enough). Terms such as `ETag` or HTTP-01 are the
standard names of the mechanisms; skip them until you need them.

- **Static files.** `Content-Type`, `ETag` and `304`, `Range`, `HEAD`, `index.html` for a directory (and a redirect from `/docs` to `/docs/`).
  A path that leaves the folder is a `404`, the same as a missing file, whether it is spelled `..`, `%2e%2e%2f`, with a backslash, as a drive or
  stream name (`C:`, `file::$DATA`), or as a symlink out of the folder. Names starting with a dot (`.git`, `.env`) are not served unless you ask
  with `.dotfiles(true)` / `--dotfiles`. Windows device names (`CON`, `NUL`, `COM1`) are refused on every system.
- **HTTPS from a certificate you already have.** `.tls_pem_files("cert.pem", "key.pem")` or `--cert cert.pem --key key.pem`, served by HTTPAnt's
  rustls acceptor (TLS 1.2 and 1.3). A certificate and key that do not belong together are refused at start, and the error never shows the key.
- **Automatic HTTPS.** `Site::new("example.org").files("./public").acme(Acme::new("you@example.org").staging())` (or
  `webservant serve --files ./public --name example.org --acme you@example.org --staging`) gets a certificate from Let's Encrypt, answers the
  HTTP-01 challenge on port 80, serves HTTPS on port 443, redirects port 80 to it, and renews before expiry with no restart (a failed renewal keeps
  the current certificate and reports the failure). `RunningSite::served_certificate(name)` reads back what the HTTPS listener really presents
  (the proof; `status()` only says what we put in service), `Site::on_tls_event` reports refused handshakes and a renewal taking effect, and
  `SIGTERM` (`systemctl stop`, `docker stop`) stops the server cleanly, as Ctrl-C does. **Run `webservant check` first** (it always uses staging): production has rate limits that a DNS or port mistake
  can use up, and staging certificates are not trusted by browsers. A name that cannot get a certificate stops the start with a sentence that says
  which step failed; the site never falls back to plain HTTP. The account and certificates are kept in your user data directory (private keys: made readable by you alone, mode 0700 on Unix and
  an `icacls`-restricted ACL on Windows, before anything is written; `--acme-store DIR` to choose another).
- **Reverse proxy.** `.proxy("/api", "http://127.0.0.1:3000")` (or `--proxy /api=http://127.0.0.1:3000`, repeatable) sends `/api/...` to your app
  with the prefix removed, next to the files. The client's `X-Forwarded-*` and `Forwarded` headers are dropped and replaced with what this server
  established, hop-by-hop headers stop here, bodies stream both ways, and an upstream that is down is a `502` (`504` on a timeout) that does not say
  where it is. WebSocket upgrades are refused with `501` in this release; only the common methods are forwarded. The timeouts (connect 5 s,
  response head 60 s, quiet body 60 s) are settings per route (`proxy_with`, or `*_timeout_secs` in `Settings`), and the request body limit (2 MiB) is
  `max_body_bytes` / `--max-body`: it applies to the whole site, so raise it for an upload endpoint knowing it raises it everywhere.
  `--proxy /=http://127.0.0.1:3000` (prefix `/` alone) makes your app answer everything else, with your files served first when they exist: a GET or
  HEAD for a file under `--files` comes from disk, anything else (another method, no such file, a directory with no `index.html`, a dotfile) goes to
  the app.
- **Will it work here?** `webservant check --name example.org --acme you@example.org --files ./public` (or `Site::check_reachable()`) does one real
  issuance against Let's Encrypt **staging** with a throwaway store, keeps nothing, and tells you which step failed (for example that the certificate
  authority could not reach port 80 on this machine). Run it before the real thing.
- **Configuration as data.** `Settings` (serde; a misspelled key is an error that names it) and `Site::from_settings` build the same site as the
  builder and the command line, so a settings system or a file in any serde format can drive it with no code. `configurant.toml` is the schema that lets
  [ConfigurAnt](https://crates.io/crates/configurant) fill it (layering, required keys and ranges are checked there; tested with ConfigurAnt 0.6.3).
- **The web library is a setting.** Hyper by default; after `cargo add webservant --features axum` (or actix-web, poem, salvo, warp),
  `.adapter("axum")` picks another and nothing else changes. Without the feature, the error says to turn it on.
- **Errors say what to do next.** Every message ends with the fix.

## What it does not do yet

The TLS-ALPN-01 and DNS-01 challenges (and so wildcard certificates), issuing a certificate on demand for a name nobody configured, WebSocket forwarding (planned, not in this release), compression, directory listings and anything for visitors to sign in with are not in this release.
The plan and the reasons are in [ROADMAP.md](https://github.com/ThinkersJournal/webservant/blob/main/ROADMAP.md) and [docs/DESIGN.md](https://github.com/ThinkersJournal/webservant/blob/main/docs/DESIGN.md).

## Status

Pre-1.0: a breaking change bumps the second number (0.2 to 0.3). Built on [HTTPAnt](https://crates.io/crates/httpant). Licence: MIT or Apache-2.0.
