# WebServAnt

**Serve a folder of files as a website, on the Rust web library you choose.**

```rust
#[tokio::main]
async fn main() -> webservant::Result<()> {
    webservant::Site::new("example.org").files("./public").serve().await
}
```

That serves `./public` at `http://127.0.0.1:8080/` until Ctrl-C. It listens on this machine only, so nothing is exposed until you say so.
The same thing without writing code:

```sh
cargo install webservant
webservant serve --files ./public
```

[`examples/hello.rs`](examples/hello.rs) is a version that stops by itself: it serves a file, fetches it and checks it. CI runs it, so a run that
exits with status 0 worked. Add it to your project with `cargo add webservant` and `cargo add tokio --features macros,rt-multi-thread`.

## What this release does

- **Static files.** `Content-Type`, `ETag` and `304`, `Range`, `HEAD`, `index.html` for a directory (and a redirect from `/docs` to `/docs/`).
  A path that leaves the folder is a `404`, the same as a missing file, whether it is spelled `..`, `%2e%2e%2f`, with a backslash, as a drive or
  stream name (`C:`, `file::$DATA`), or as a symlink out of the folder. Names starting with a dot (`.git`, `.env`) are not served unless you ask
  with `.dotfiles(true)` / `--dotfiles`. Windows device names (`CON`, `NUL`, `COM1`) are refused on every system.
- **HTTPS from a certificate you already have.** `.tls_pem_files("cert.pem", "key.pem")` or `--cert cert.pem --key key.pem`, served by HTTPAnt's
  rustls acceptor (TLS 1.2 and 1.3). A certificate and key that do not belong together are refused at start, and the error never shows the key.
- **Automatic HTTPS.** `Site::new("example.org").files("./public").acme(Acme::new("you@example.org").staging())` (or
  `webservant serve --files ./public --name example.org --acme you@example.org --staging`) gets a certificate from Let's Encrypt, answers the
  HTTP-01 challenge on port 80, serves HTTPS on port 443, redirects port 80 to it, and renews before expiry with no restart (a failed renewal keeps
  the current certificate and reports the failure). **Do the first run with `--staging`**: production has rate limits that a DNS or port mistake
  can use up, and staging certificates are not trusted by browsers. A name that cannot get a certificate stops the start with a sentence that says
  which step failed; the site never falls back to plain HTTP. The account and certificates are kept in your user data directory (private keys:
  `--acme-store DIR` to choose another).
- **Reverse proxy.** `.proxy("/api", "http://127.0.0.1:3000")` (or `--proxy /api=http://127.0.0.1:3000`, repeatable) sends `/api/...` to your app
  with the prefix removed, next to the files. The client's `X-Forwarded-*` and `Forwarded` headers are dropped and replaced with what this server
  established, hop-by-hop headers stop here, bodies stream both ways, and an upstream that is down is a `502` (`504` on a timeout) that does not say
  where it is. WebSocket upgrades are refused with `501` in this release; only the common methods are forwarded.
- **The web library is a setting.** Hyper by default; after `cargo add webservant --features axum` (or actix-web, poem, salvo, warp),
  `.adapter("axum")` picks another and nothing else changes. Without the feature, the error says to turn it on.
- **Errors say what to do next.** Every message ends with the fix.

## What it does not do yet

The TLS-ALPN-01 challenge (waiting on HTTPAnt: its `ServerCert` rejects the RFC 8737 validation certificate), DNS-01 and wildcards, compression, directory listings and anything for visitors to sign in with are not in this release.
The plan and the reasons are in [ROADMAP.md](ROADMAP.md) and [docs/DESIGN.md](docs/DESIGN.md).

## Status

Pre-1.0: a breaking change bumps the second number (0.2 to 0.3). Built on [HTTPAnt](https://crates.io/crates/httpant). Licence: MIT or Apache-2.0.
