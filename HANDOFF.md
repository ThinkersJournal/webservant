# WebServAnt lane HANDOFF

Current state only; history is in git. Rewritten 2026-11-16 by the lane. Worktree C:\Projects\webservant-wt-lane (branch lane/webservant-0.4.13, local until the PR). Repo nwo ThinkersJournal/webservant. Primary checkout stays on main. PUBLISHED: 0.4.12 on crates.io (PM, tag v0.4.12, one release for 0.4.5-0.4.12).

## State
Everything the roadmap names is built and merged (#1 to #23). Static files, HTTPS from PEM, ACME HTTP-01 (Pebble in Linux CI), renewal, reverse proxy, check_reachable, Settings, adapters, QUICKSTART, patent sweep.

## In flight
- PR 0.4.13 (docs only): README/ROADMAP/CONSUMER-CONTRACT say "on crates.io at 0.4.12"; Part 1 re-run from the registry (recorded in CHANGELOG). PM allocates the number. On merge: delete branch via API, `git worktree`/branch cleanup per CLAUDE.md section 3.
- Told pm-ants (webhostant `local` can take 0.4.12).

## Honest gaps (claim no more)
- No person has followed QUICKSTART Part 2 (board 455). Staging runs were by an AI agent. Never run: production, failed renewal on a real CA, reboot with the systemd unit, phone on mobile data. Part 2 not re-run from the registry.
- Connect timeout validated, untested; ConfigurAnt env/flag layers unexercised; no `--config FILE`.
- Out of first version (re-sweep first): on-demand issuance, DNS-01, TLS-ALPN-01, WebSocket forwarding.

## Rules that bit here
- I propose versions, the PM allocates; one PR per day per project.
- Squash-merge: rebase with `git rebase --onto origin/main <old-base-tip> <branch>`; a stale sequencer can block `git switch` (`git cherry-pick --quit`).
- Shell eats backslashes/apostrophes in heredocs; files are CRLF (check line endings after Python edits). Sign every PR body/comment last line "— lane: webservant". Sonnet only for subagents. Never `gh auth switch`. Untracked `supertool` is not ours: `git add -A . ':!supertool'`.

## Next
1. Open the PR, hand to gate ([READY]); fix findings.
2. After merge: cleanup, then restart.

WAIT: PM merge of the 0.4.13 PR, or a [TASK] | check: git fetch -q && git log --oneline origin/main -3; synapse fetch | wake: 2026-11-17T04:00:00Z
