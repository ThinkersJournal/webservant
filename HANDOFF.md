# WebServAnt lane HANDOFF

Current state only; history is in git. Written 2026-11-08 by the lane.

## Why (CireSnave 2026-10-28, ROADMAP.md): build order HTTPAnt -> WebServAnt -> WebHostAnt local serving. HTTPAnt 0.5.22 is published.

## State
- Merged: #1 design, #2 slice 1 (0.1.0: static files, HTTPS from a cert pair, CLI). PM rulings: D1/D2/D5/D8 as written; the patent sweep (row 5) and a Let's Encrypt STAGING run must be done before the first PUBLISH (not before merge); nothing touches Let's Encrypt production before a staging run is recorded in a PR.
- PR #3, branch slice2-acme in C:\Projects\webservant-wt-lane: ROADMAP row 2 built (src/acme.rs, `Acme` in lib.rs, CLI --acme/--staging/--acme-store/--http-bind/--also-name). HTTP-01 only. Version 0.2.0 is a PROPOSAL (Site lost Clone: breaking pre-1.0); PM allocates.
- Evidence: 7 fake-CA tests; Pebble-in-Docker test of the real protocol (passes locally; the Linux CI step requires it via WEBSERVANT_REQUIRE_PEBBLE, its first GitHub run is pending: watch it); clippy -D warnings, doc, deny clean. Mutants caught: challenge not published, due at half lifetime, failed renewal drops the cert, unknown or absent SNI served.
- NOT done: a run against Let's Encrypt staging (needs a public name + port 80: ask the PM); Windows ACL on the store; TLS-ALPN-01, blocked on HTTPAnt (ServerCert::from_pem/from_der reject a cert with a CRITICAL acmeIdentifier extension; critical=false and no-extension pass; [FINDING] sent to the PM).
- Untracked `supertool` file is not ours.

## Next
1. Watch PR #3 CI and threads; fix findings.
2. Then row 3 (reverse proxy), row 5 (patent sweep, needed before publish), row 4 (WebHostAnt contract; check with the WebHostAnt lane before freezing).

WAIT: slice-2 PR gate and merge | check: gh pr list --repo ThinkersJournal/webservant --state merged --head slice2-acme --json number,mergedAt | wake: 2026-11-08T09:00:00Z
