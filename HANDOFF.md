# WebServAnt lane HANDOFF

Current state only; history is in git. Written 2026-11-08 by the lane.

## Why (CireSnave 2026-10-28, ROADMAP.md): build order HTTPAnt -> WebServAnt -> WebHostAnt local serving. HTTPAnt 0.5.22 is published.

## State
- Merged: #1 design, #2 slice 1 (0.1.0), #3 slice 2 ACME HTTP-01 (0.2.0). PM rulings: the patent sweep (row 5) and a Let's Encrypt STAGING run (needs CireSnave's public name + port 80; the PM asked him) must be done before the first PUBLISH; nothing touches Let's Encrypt production before a staging run is recorded in a PR. HTTPAnt acmeIdentifier fix is with the HTTPAnt lane (TLS-ALPN-01 waits on it).
- Branch slice3-proxy in C:\Projects\webservant-wt-lane: ROADMAP row 3 built (src/proxy.rs, `Site::proxy`, CLI --proxy). Version 0.2.1 is a PROPOSAL; PM allocates. 9 integration + 3 unit proxy tests; mutants caught: forwarding headers, hop-by-hop, Connection-named, upgrade. Survivors, stated: the dot-segment check and the prefix-boundary check are second layers behind HTTPAnt's router (unit tests only).
- NOT done in row 3: WebSocket upgrade (501), per-route timeouts/body limit, HTTP/2 upstream, uncommon methods.
- #4 merged (0.2.1). Branch docs-consumer-contract: docs/CONSUMER-CONTRACT.md (row 4, 0.2.2 proposed, unreviewed: the WebHostAnt lane is suspended until CireSnave says yes). PR open.
- Untracked `supertool` file is not ours.

## Next
1. Watch the row-4 doc PR (CI, threads); on merge start row 5.
2. Then row 5 (patent sweep: needed before publish; real search work), row 4 (WebHostAnt consumer contract: ask the WebHostAnt lane before freezing), Windows ACL on the store, TLS-ALPN-01 once HTTPAnt's fix is published.

WAIT: row-4 doc PR gate and merge | check: gh pr list --repo ThinkersJournal/webservant --state merged --head docs-consumer-contract --json number,mergedAt | wake: 2026-11-08T10:00:00Z
