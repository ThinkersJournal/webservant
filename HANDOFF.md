# WebServAnt lane HANDOFF

Current state only; history is in git. Rewritten 2026-11-11 by the lane. Worktree C:\Projects\webservant-wt-lane on branch lane/webservant (= origin/main
at 695837b, 0.4.6; the docs PR for run 4c proposes 0.4.7). The primary checkout C:\Projects\webservant stays on main. 0.4.4 is PUBLISHED (tag v0.4.4).

## Why (CireSnave 2026-10-28, ROADMAP.md)
Build order HTTPAnt -> WebServAnt -> WebHostAnt local serving. HTTPAnt 0.7.1 is published and depended on as `~0.7.1` (PM ruling D2, 2026-11-10; merged in #17, 0.4.6).

## State: everything the roadmap names is built and merged (#1 to #17, 0.4.6); 0.4.4 is the published version
Built: static files (traversal/symlink/range/304 tests), HTTPS from a PEM pair, ACME HTTP-01 certificates via `instant-acme` behind our `AcmeClient`
(fake CA + Pebble in Docker, required in Linux CI), renewal that keeps the last good certificate, private store (Unix 0700, Windows icacls), reverse
proxy (prefix and `/` with files first, per-route timeouts, site-wide body limit), `check_reachable` / `webservant check`, `Settings` + ConfigurAnt
schema (file layer tried with ConfigurAnt 0.6.3), live read-back, TLS events, SIGTERM, QUICKSTART, three fresh-eyes runs, patent sweep.

Waiting on people, not on code:
- **CireSnave** (via the PM): board 402, whether the thin patent sweep (`docs/PATENT-SWEEP.md`, 14 hits read, 8 queries, thin by its own
  Limitations) meets the first-publish bar; and the Let's Encrypt STAGING run on his VPS (`webservant check`; needs his public name and port 80),
  where the PM will ask him to follow `docs/QUICKSTART.md` Part 2 verbatim and send every failure. **Every failure he sends goes into QUICKSTART and
  the trial record.** Nothing may touch Let's Encrypt production before a staging run is recorded in a PR.
- **The PM**: version allocation (always: I only propose), merge, publish. The PM publishes only after the sweep decision and the staging run.
- **HTTPAnt lane**: the fix for `ServerCert` rejecting a critical `acmeIdentifier` extension (TLS-ALPN-01 is OUT of the first version anyway).

Keep OUT of the first version (each needs a patent re-sweep first): on-demand issuance, DNS-01, TLS-ALPN-01, WebSocket forwarding.

## Honest gaps (do not claim more than this)
- No agent or person has RUN QUICKSTART Part 2 on a real Linux server. The three fresh-eyes runs only read it (Windows; they were forbidden from
  touching a real CA). The last round of fixes after run 3 was not re-tried.
- On Let's Encrypt STAGING, Debian 13 VPS, run by the PM's agent over SSH (trial runs 4, 4b, 4c): `check`, `serve`, reuse, renewal at start and renewal on a RUNNING server (same PID) all passed. Never run: a failed renewal on a real CA (tests only), production, the systemd unit (steps sent to the PM 2026-11-11), the phone-on-mobile-data step, a person following the page.
- 0.4.4 is PUBLISHED (PM, 2026-11-10, tag v0.4.4 at ee5a23a). `cargo install webservant --version 0.4.4` was run on Windows and worked; not run on Linux. README is marked pre-release and the quickstart block is deleted (PR #16).
- The connect timeout is validated but untested; ConfigurAnt's environment and flag layers are not exercised; no `--config FILE` (needs a TOML dep).
- Not built, only if asked: add a name to a running site (C1); per-route body limit (HTTPAnt's limit is site-wide).

## Rules that bit here
- Version: I propose, the PM allocates; every pushed change bumps (pre-1.0 the second number is "major"). Docs-only took 0.x.1+ too.
- Squash-merge: rebase the next branch with `git rebase --onto origin/main <old-base-tip> <branch>`.
- Shell quoting on this box: heredocs and python strings eat backslashes and apostrophes; write scripts with the Write tool, edit with Edit, and
  anchor Python replacements on text without backslashes. A shell command that fails to PARSE runs none of its lines; I once recorded an edit as
  done that had never run. Read the file back before claiming a fix.
- CI flakes seen: cargo-deny action's docker `apk update`, and CodeRipper's gitleaks download (HTTP 504) fail the portfolio job by design when a rule
  cannot run. Re-run only the failed jobs; read the log first.
- End every GitHub PR body, comment and squash-merge note with a last line "— lane: webservant" (CireSnave 2026-11-09). Sonnet only for subagents; short messages; headers per MESSAGE-FORMATS.md. Never `gh auth switch`. An untracked file `supertool` in the worktree
  is not ours: never add it (`git add -A . ':!supertool'`).

## Next
1. On start: `git fetch`; `lane-restart waits`; check synapse for the PM's relay of CireSnave's results.
2. If he sent QUICKSTART failures: fix the page, add them to docs/FRESH-EYES-TRIAL.md (as a person's run, not an agent's), PR, [READY].
3. Otherwise idle. Do not invent work: the roadmap is done up to what needs a person.

UPDATE 2026-11-08: board 402 RULED (CireSnave: "402: A"): publish the first version on our sweep once the staging run is done and httpant 0.7.1 is on
crates.io (PM publishes it next; our `~0.5.22` dep will need bumping then, PM allocates the version). Four gated features stay out until a re-sweep.

WAIT: PM output of the systemd-unit run (QUICKSTART step 8, --staging; steps sent 2026-11-11) and PM gate of the docs PR recording run 4c | check: git fetch -q && git log --oneline origin/main -3 | wake: 2026-11-11T11:00:00Z
