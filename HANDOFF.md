# WebServAnt lane HANDOFF

Current state only; history is in git. Written 2026-11-08 by the lane.

## Why (CireSnave 2026-10-28, ROADMAP.md): build order HTTPAnt -> WebServAnt -> WebHostAnt local serving. HTTPAnt 0.5.22 is published.

## State
- Merged: #1 design, #2 (0.1.0), #3 ACME HTTP-01 (0.2.0), #4 proxy (0.2.1), #5 consumer contract + patent sweep (0.2.2). PM: whether the thin sweep meets the first-publish bar is CireSnave's (board 402); a Let's Encrypt STAGING run is also needed before publish (CireSnave's public name + port 80). Keep OUT of the first version: on-demand issuance, DNS-01, TLS-ALPN-01, WebSocket (each needs a re-sweep first).
- Branch slice4-contract-gaps (0.3.0 proposed, breaking: ALPN code removed): TLS-ALPN-01 code removed; C6 `RunningSite::served_certificate`; C5 `Site::on_tls_event`; C11 `shutdown_signal()` + SIGTERM. Tests: 2 new in acme_fake.rs (mutant on the event sink caught), tests/signal.rs is unix-only and has NOT run locally (Windows): its first run is Linux CI, watch it.
- Remaining contract gaps: C10 settings struct (serde, for ConfigurAnt), C2 check_reachable(), C1 add a name to a running site (only if asked), Windows ACL on the store, 504/timeouts as settings, proxy at "/" next to files.
- Untracked `supertool` file is not ours.

## Next
1. Watch the slice-4 PR (CI incl. Linux signal test, threads).
2. Then C10 settings struct, then C2.

WAIT: slice-4 PR gate and merge | check: gh pr list --repo ThinkersJournal/webservant --state merged --head slice4-contract-gaps --json number,mergedAt | wake: 2026-11-08T11:00:00Z
