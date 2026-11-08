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
- **The web library is a setting.** Hyper by default; after `cargo add webservant --features axum` (or actix-web, poem, salvo, warp),
  `.adapter("axum")` picks another and nothing else changes. Without the feature, the error says to turn it on.
- **Errors say what to do next.** Every message ends with the fix.

## What it does not do yet

Automatic certificates (ACME), a reverse proxy, compression, directory listings and anything for visitors to sign in with are not in this release.
The plan and the reasons are in [ROADMAP.md](ROADMAP.md) and [docs/DESIGN.md](docs/DESIGN.md).

## Status

Pre-1.0: a breaking change bumps the second number (0.1 to 0.2). Built on [HTTPAnt](https://crates.io/crates/httpant). Licence: MIT or Apache-2.0.
