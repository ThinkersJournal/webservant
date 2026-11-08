# WebServAnt lane HANDOFF

Current state only; history is in git. Written 2026-11-08 by the lane.

## Why (CireSnave 2026-10-28, ROADMAP.md): build order HTTPAnt -> WebServAnt -> WebHostAnt local serving. HTTPAnt 0.5.22 is published.

## State
- Merged: #1 to #8 (0.3.2: `Site::check_reachable()` / `webservant check`). PM: whether the thin sweep meets the first-publish bar is CireSnave's (board 402); a Let's Encrypt STAGING run is also needed before publish (`webservant check` on his VPS; needs his public name + port 80). Keep OUT of the first version: on-demand issuance, DNS-01, TLS-ALPN-01, WebSocket (each needs a re-sweep first).
- PR for slice7-configurant (0.3.3 proposed): `configurant.toml` schema + tests/configurant.rs (ConfigurAnt 0.6.3 as a dev-dependency fills `Settings`; file layer only). Finding: ConfigurAnt's `deserialize` builds EMPTY intermediate tables for every declared prefix, so `Option<struct>` fields come back `Some(default)`; we now treat an all-default `acme`/`tls` table as absent (mutant caught). [FINDING] sent to the PM for the ConfigurAnt lane.
- Remaining: environment/flag layers through ConfigurAnt not exercised; `--config FILE` (TOML dependency); C1 add a name to a running site (only if asked); Windows ACL on the store; proxy timeouts as settings; proxy at "/" next to files; no run against Let's Encrypt staging.
- Untracked `supertool` file is not ours.

## Next
1. Watch the slice-7 PR (CI, threads).
2. Then Windows ACL on the store (test with icacls), proxy timeouts/limits as settings.

WAIT: slice-7 PR gate and merge | check: gh pr list --repo ThinkersJournal/webservant --state merged --head slice7-configurant --json number,mergedAt | wake: 2026-11-08T14:00:00Z
