# WebServAnt lane HANDOFF

Current state only; history is in git. Written 2026-11-08 by the lane.

## Why (CireSnave 2026-10-28, ROADMAP.md): build order HTTPAnt -> WebServAnt -> WebHostAnt local serving. HTTPAnt 0.5.22 is published.

## State
- Merged: #1 to #12 (0.4.1: proxy at / with files first). PM: whether the thin sweep meets the first-publish bar is CireSnave's (board 402); a Let's Encrypt STAGING run is also needed before publish (`webservant check` on his VPS; needs his public name + port 80). Keep OUT of the first version: on-demand issuance, DNS-01, TLS-ALPN-01, WebSocket (each needs a re-sweep first).
- PR for slice11-quickstart (0.4.2 proposed): docs/QUICKSTART.md, docs/FRESH-EYES-TRIAL.md (run 1: Part 1 passed first try; 13 findings all fixed; Part 2 only READ, never run; no second run yet), CLI `--help`/`--version`/did-you-mean, `--acme-directory`/`--acme-ca-root`, Pebble test of the real `webservant check` output sentences, doc-command parse test, packaging (`include`, real description).
- Remaining: a SECOND fresh-eyes run on the fixed pages (HTTPAnt had three; costs an agent, ~65k tokens, ~7 min); nobody has run Part 2 on a real Linux server; `cargo install webservant` from crates.io untested; connect timeout untested; ConfigurAnt env/flag layers not exercised; `--config FILE` (TOML dependency); C1 add a name to a running site (only if asked); no run against Let's Encrypt staging; delete the "until the first release" block in QUICKSTART at publish.
- Untracked `supertool` file is not ours.

## Next
1. Watch the slice-11 PR (CI, threads). Then idle (restart rule) unless the PM has more.

WAIT: slice-11 PR gate and merge | check: gh pr list --repo ThinkersJournal/webservant --state merged --head slice11-quickstart --json number,mergedAt | wake: 2026-11-08T18:00:00Z
