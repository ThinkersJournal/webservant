# WebServAnt lane HANDOFF

Current state only; history is in git. Written 2026-11-08 by the lane.

## Why (CireSnave 2026-10-28, ROADMAP.md): build order HTTPAnt -> WebServAnt -> WebHostAnt local serving. HTTPAnt 0.5.22 is published.

## State
- Worktree C:\Projects\webservant-wt-lane, branch lane/webservant. PR ThinkersJournal/webservant#1 (head ef7118b): docs/DESIGN.md draft + ROADMAP rows, docs only.
- PM ruled D1, D2, D5, D8 as written (2026-11-08). Slice 1 starts after #1 merges. Adopt instant-acme behind AcmeClient. Nothing may run against Let's Encrypt production until a staging run is recorded in the PR.
- Untracked `supertool` file in the worktree is not ours; leave it out of commits.

## Next
1. Check PR #1 (gh pr view 1 --json state,statusCheckRollup; unresolved review threads via GraphQL) and any PM reply (synapse fetch).
2. On answers (or silence past the wake time, assume the recommendations and say so to the PM), start row 1 red-first: path-traversal/symlink/range/304/Windows-name tests, then the handler; one PR per slice; [READY] after each.
3. Unchecked in DESIGN.md (marked UNVERIFIED): instant-acme pluggable HTTP client, Pebble on this box, tower layer fit, HTTPAnt client HTTP/1.1 only.

WAIT: PR #1 gate and merge (then start slice 1) | check: gh pr view 1 --repo ThinkersJournal/webservant --json state,mergedAt | wake: 2026-11-08T05:18:32Z
