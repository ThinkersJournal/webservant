# WebServAnt lane HANDOFF

Current state only; history is in git. Rewritten 2026-11-26 by the lane. Worktree C:\Projects\webservant-wt-lane. Repo ThinkersJournal/webservant. Primary checkout stays on main. Publishing is the PM's.

## State
Roadmap built and merged (#1 to #26). PR #27 (branch `lane/webservant-sigterm-audit`, docs only) is open and in the gate; READY sent to pm-assist cc pm-ants 2026-11-27. Version is the gate's. When #27 merges: delete branch, `git worktree remove`, start a fresh branch.

## WAITs (each audit row blocked; arm on restart)
- WAIT webservant#27 merge: check `gh pr view 27 --json state`; then cleanup above.
- WAIT elevated-session-or-Developer-Mode (CireSnave): Windows file-symlink escape test needs Administrator (NewItemSymbolicLinkElevationRequired); runs in Linux CI meanwhile.
- WAIT host (CireSnave): systemd unit, real-CA ACME, production, phone on mobile data: never run, no host.

## Honest gaps (claim no more)
- No person has followed QUICKSTART Part 2 (board 455). Never run: production, failed renewal on a real CA, systemd reboot, phone on mobile data.
- Connect timeout validated, untested; ConfigurAnt env/flag layers unexercised; no `--config FILE`.
- Out of first version (re-sweep first): on-demand issuance, DNS-01, TLS-ALPN-01, WebSocket forwarding.

## Rules that bit here
- I propose versions, PM allocates. Squash-merge: `git rebase --onto origin/main <old-base-tip> <branch>`.
- Files are CRLF; sign PR text "— lane: webservant". Sonnet only. Never `gh auth switch`. Untracked `supertool` is not ours: never `git add -A`/`commit -a`; add by path.
- CLI flags: `--bind` (not `--addr`).
