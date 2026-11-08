# Changelog

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
