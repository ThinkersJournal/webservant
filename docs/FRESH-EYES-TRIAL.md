# Fresh-eyes trial of the quickstart

EXPECTATIONS 6.24: *an agent given only the README must reach the result* is part of the first-release gate. Method as HTTPAnt's
`docs/FRESH-EYES-TRIAL.md`: the packaged crate behind an offline directory source, a fresh agent that knows nothing else, its report quoted.
**HTTPAnt's gate had three runs. This file records run 1, run 2 on the fixed pages, and run 3 if it appears below.**

## Run 1, 2026-11-08

A fresh `sonnet` subagent (no memory of the project), Windows 11, Git Bash, offline, at `slice11-quickstart` (0.4.2). **Result: Part 1 reached
the promised result on the first attempt, both ways; Part 2 could not be executed and was read.** The pages had 13 problems (below), all fixed
afterwards; none of the fixes has been re-tried by a second agent.

### What the agent was given, and what it was not

- `README.md` and `docs/QUICKSTART.md` exactly as the packaged crate ships them (`cargo package`; the README was checked byte-identical with the
  repository's with `cmp`), and nothing else about the project.
- A cargo configuration that replaces crates.io with a **directory source** holding the 288 crates `cargo vendor` produced for the dependency tree
  plus `webservant` exactly as `cargo package` made it (checksum of the `.crate` recorded). `cargo add webservant` resolves to the packaged 0.4.2.
- **`cargo install webservant` does not work against a directory source in this setup** ("could not find `webservant` in registry `crates-io`"), so the
  agent was told to install with `cargo install --path <packaged crate>`. That compiles the packaged sources but is not the command a stranger types.
- Rules: use only those two files, do not read other folders or search the web. **It was forbidden to run `webservant check` with real flags or
  `serve --acme`**, because they would contact a real certificate authority about names nobody here controls. So **Part 2's steps were not executed
  and none of the Part 2 commands was tried by the agent.**

### Limits of the method

- The agent was told, not prevented, from reading elsewhere. Its report lists no reads outside its folder.
- A directory source simulates publication; the crates.io page, docs.rs and download time are not exercised.
- One agent, one run, one platform (Windows). An AI is a proxy for a stranger with Rust knowledge a newcomer lacks. **Part 2 is written for Linux and
  was only read, on Windows, by someone who could not run it.** Nothing in this trial shows that Part 2 works on a real server. The evidence for its
  literal text is the Pebble test of the `check` command's two output sentences (tests/acme_pebble.rs) and the test that parses every documented
  command; no run against Let's Encrypt, staging or production, has happened.

### What the agent reported (model output, condensed; commands and quotes as it gave them)

- Part 1, command-line way: installed (about 1m30s of compiling), `webservant serve --files ./public` printed exactly
  `serving http://127.0.0.1:8080/  (Ctrl-C to stop)`; `curl` got `200` and the page. Rust way: `cargo add webservant` resolved, build 1m51s, page
  served. Both on the first attempt.
- Part 1 failure rows provoked: missing folder (matches), no `--files` (not in the table), two copies on one `--bind` (the message was Windows'
  `Only one usage of each socket address ... (os error 10048)`, not the table's "address already in use"; the advice worked), a typo (matched, but
  "the output is a 7-line wall of usage and the bad flag is only on line 1"), `--help` ("unknown option `--help` ... which is hostile").
- Part 2, read only. Its answers matched the page for which two ports and why, the exact test command and its success line, and what to check first
  for each failure row. Things it said it could not do from the page alone: the `setcap $(which webservant)` and the systemd unit's
  `/usr/local/bin/webservant` do not agree with where `cargo install` puts the binary; no way to check outside reachability before the real test; no
  `AAAA` check command; whether the email must be real; the Windows hand-wave.
- Verdict: "A non-expert who already has Rust installed can reach the Part 1 result from the page in one attempt ... Single biggest friction: waiting
  [about 4 to 5 minutes of silent compiling, and the Rust way printing nothing when it works so you cannot tell success from a hang]. Second: the stale
  'Honest note' at the top ... without giving a repository URL."

### The 13 findings, and what was done about each

| # | finding | done |
|---|---|---|
| 1 | The note that the install command may not work yet gave no repository address and read as alarming | now a short, plain block with the `git clone` and `cargo install --path .` lines; to be deleted at release |
| 2 | The Rust way prints nothing, so success looked like a hang; "same result" hid it | the page says so in bold, in the Rust way and in the failure table |
| 3 | "Put this in `src/main.rs`" did not say to replace the generated contents; compile time contradicted "5 minutes"; `echo` under Windows PowerShell 5 writes UTF-16 | "replace everything"; the heading and text say most of the time is compiling; "or make the file in any text editor" |
| 4 | The table promised the Linux wording "address already in use"; the "refused to connect" row's advice contradicted step 3 | both wordings listed; the row rewritten; a "nothing to serve" row added |
| 5 | A typo printed a 7-line usage wall with the flag on line 1; no "did you mean"; `--help` was an error | one line naming the flag with a suggestion (`Did you mean --files?`); `--help`, `-h`, `help`, `--version` work (tests in `src/main.rs`) |
| 6 | The README said `serve --files ./public` without creating `./public` | the README now creates it |
| 7 | The README was dense with unexplained jargon ("each needs a new patent re-sweep first" was baffling) | a plain lead-in saying the list is detail to skip; the patent wording removed from the README. **Run 2 found that one sentence, "each is explained where it matters in the quickstart", was still there and false** (the edit had been in a command that failed to run, and I had recorded it as done); fixed after run 2 |
| 8 | "prints one line starting with NOT REACHABLE or `webservant:`" was false for a typo (several lines, no prefix) | the lead-in now says the messages start in a few different ways; the typo row names its start |
| 9 | Step 7 read as though `check` took `--staging`; the README and page disagreed about when to use staging | step 6 says `check` always uses staging; step 7 says it uses the real service and why you tested first; the README now says "Run `webservant check` first" |
| 10 | The Windows data directory was unverified | removed from the page (Linux only is claimed) |
| 11 | "step 2" and "step 6" were ambiguous between Part 1 and Part 2 | Part 1 has no numbered cross-references; Part 2's steps are referred to as "steps N" inside Part 2 only |
| 12 | The README said CI runs `examples/hello.rs` and told a crates.io user to "add it" | reworded: the example exists for the repository's CI, and the library use is `cargo add` |
| 13 | A newcomer's terminal may not see the new binary | the page says to open a new terminal if the command is not found |

Also found by the agent and fixed: `cargo install` puts the binary in `~/.cargo/bin` while the `setcap` and systemd instructions assumed
`/usr/local/bin`; step 4 now copies it there and applies `setcap` to the copy. And the missing outside-reachability check became step 5 (a plain HTTP
server on port 80 and a phone on mobile data; that step has not been run by anyone).

### What this trial does not show

That Part 2 works. That a Linux newcomer succeeds. That the fixed pages are better (no second run). That `cargo install webservant` works from
crates.io. One of the agent's compound commands, which included a valid `check` without `--acme`, was stopped by the harness before it ran; nothing
was sent to any certificate authority.

## Run 2, 2026-11-08, on the pages as fixed after run 1

A second fresh `sonnet` agent, same setup (the packaged crate rebuilt from the fixed branch, a new sandbox folder, the first run's installed binary
removed from the shared cargo bin first so that nothing pre-answered it), same rules, same tasks, plus two asks: try to get help from the program as
a newcomer would, and read Part 2 commands-first for contradictions between steps. It was not told what run 1 found. Again it did not run `check`
with real flags or `--acme`. Its install was again `cargo install --path` (a directory registry cannot do `cargo install webservant`).

**Result: Part 1 on the first attempt both ways again; every Part 1 failure row matched what it saw; `--help`, `help`, `serve --help` and
`--version` worked; the typo printed one line with the suggestion (`unknown option `--fles` ... Did you mean `--files`?`).** Its answers to the
Part 2 questions (ports and why, the test command and its success line, what to check first per failure row) were again right. It found **no problem
in Part 1's flow** except the install story and two small things, and 12 in Part 2 and the README, all from reading:

| # | finding (run 2) | done |
|---|---|---|
| 1 | "It is installed to ~/.cargo/bin" was wrong on its machine (CARGO_HOME elsewhere) | the page says cargo prints the path on its last line, and `~/.cargo/bin` is only "normally" |
| 2 | The install-from-source box did not flow into the steps (where to `mkdir public` after `cd webservant`; steps still said `cargo install webservant`) | the box says "wherever this page says `cargo install webservant`, do this instead", and ends with `cd ..` |
| 3 | `cargo add webservant` prints a wall of feature names that looks like an error; the Rust way gives no sign it is running | the page says the wall is normal and that cargo's `Running ...` line is the last thing you see |
| 4 | PowerShell 5 / cmd and `echo` | "use a text editor in PowerShell or cmd" |
| 5 | README: "That serves ./public ..." came right after the code with no context; the sentence "each is explained where it matters in the quickstart" was false | "The program above serves ..."; the sentence now says to skip the terms until needed |
| 6 | Part 2 step 5's server must be stopped before step 6 (port 80), whether or not step 5 worked | "Either way, press Ctrl-C ... before step 6" |
| 7 | Two copies of the binary (`~/.cargo/bin` first on PATH, no `setcap`) would give "Permission denied" | step 4 ends with `cargo uninstall webservant` so only the permitted copy remains; "repeat the three lines after an upgrade" |
| 8 | `cargo install` needs a C compiler on a fresh server | listed in "What you need" with the Ubuntu package |
| 9 | The systemd unit used `/srv/site` but nothing copied `public` there; no `daemon-reload`; a second, empty store would make a second order | copy and reload commands added; the page says the service has its own store and not to alternate many times |
| 10 | `ufw` without allowing SSH locks you out | warning added |
| 11 | "0.0.0.0" in the success URL; the AAAA check's "no answer" wording; "About 20 minutes" vs DNS taking hours | each explained; the time is "20 minutes of work plus waiting for DNS" |
| 12 | `--bind foo` gets the advice "if the address is in use ... `.bind(...)`" (Rust API wording) | the message now also gives the command-line form `--bind 127.0.0.1:8081` |

Verdict of the agent: "Part 1 is followable by a non-expert: it worked first try both ways, every error message matched the table ... The single
biggest friction is the install story ... For Part 2, the biggest risk is the binary-copy / setcap / PATH confusion and leaving the step 5 server
running before step 6."

What run 2 does not show is the same list as run 1's: Part 2 was read, not run; one platform; an AI as the stranger.
