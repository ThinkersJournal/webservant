# WebServAnt lane HANDOFF

Current state only; history is in git. Rewritten 2026-11-26 by the lane. Worktree C:\Projects\webservant-wt-lane. Repo ThinkersJournal/webservant. Primary checkout stays on main. Publishing is the PM's.

## State
Roadmap built and merged (#1 to #26). Branch `lane/webservant-sigterm-audit` (from origin/main d24cdf73) has 3 commits, pushed, NO PR yet: README SIGTERM (Linux/container), HANDOFF, adapters + Settings audit (registry 0.4.13 crate, all 6 adapters OK; CHANGELOG [Unreleased]). Open ONE PR on the next UTC day after 2026-11-26; [READY] to pm-assist cc pm-ants. Version is the gate's.

## Blocked
- File-symlink escape on Windows: creating a symlink needs Administrator here (NewItemSymbolicLinkElevationRequired); runs in Linux CI. Only unblockable by an elevated session or Developer Mode (CireSnave).
- systemd unit, real-CA ACME, production: never run, no host.

## Honest gaps (claim no more)
- No person has followed QUICKSTART Part 2 (board 455). Never run: production, failed renewal on a real CA, systemd reboot, phone on mobile data.
- Connect timeout validated, untested; ConfigurAnt env/flag layers unexercised; no `--config FILE`.
- Out of first version (re-sweep first): on-demand issuance, DNS-01, TLS-ALPN-01, WebSocket forwarding.

## Rules that bit here
- I propose versions, PM allocates. Squash-merge: `git rebase --onto origin/main <old-base-tip> <branch>`.
- Files are CRLF; sign PR text "— lane: webservant". Sonnet only. Never `gh auth switch`. Untracked `supertool` is not ours: never `git add -A`/`commit -a`; add by path.
- CLI flags: `--bind` (not `--addr`).
