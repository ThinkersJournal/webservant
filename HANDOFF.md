# WebServAnt lane HANDOFF

Current state only; history is in git. Written 2026-11-08 by the lane.

## Why (CireSnave 2026-10-28, ROADMAP.md): build order HTTPAnt -> WebServAnt -> WebHostAnt local serving. HTTPAnt 0.5.22 is published.

## State
- Merged: #1 to #9 (0.3.3: Settings tried with ConfigurAnt 0.6.3, file layer). PM: whether the thin sweep meets the first-publish bar is CireSnave's (board 402); a Let's Encrypt STAGING run is also needed before publish (`webservant check` on his VPS; needs his public name + port 80). Keep OUT of the first version: on-demand issuance, DNS-01, TLS-ALPN-01, WebSocket (each needs a re-sweep first).
- PR for slice8-store-acl (0.3.4 proposed): the ACME store directory is made private before the first write (Unix 0700, test runs in Linux CI only; Windows icacls by SID with system tools by full path, tested here with a mutant caught). Fails closed: if it cannot be made private nothing is written.
- Remaining: ConfigurAnt env/flag layers not exercised; `--config FILE` (TOML dependency); C1 add a name to a running site (only if asked); proxy timeouts as settings; proxy at "/" next to files; no run against Let's Encrypt staging.
- Untracked `supertool` file is not ours.

## Next
1. Watch the slice-8 PR (CI incl. Linux 0700 test, threads).
2. Then proxy timeouts/body limit as settings; proxy at "/" next to files.

WAIT: slice-8 PR gate and merge | check: gh pr list --repo ThinkersJournal/webservant --state merged --head slice8-store-acl --json number,mergedAt | wake: 2026-11-08T15:00:00Z
