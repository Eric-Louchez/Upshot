# Upshot — instructions for Claude

This repository builds **Upshot**: a language for the data-driven aspects of development, implemented in Rust and
targeting native interface integration with C and C++.

## Where things are

| Path | What | Treat it as |
|---|---|---|
| `spec/log/` | The user's messages verbatim (L ids), append-only | The authority on *what* to build |
| `spec/current/` | The current specification by area: rules with area ids (`LANG-4`), each with a `Source:` line | What Claude builds from |
| `spec/interpretations.md` | Claude's interpretations (J table, append-only) | A source the current spec cites |
| `plan/` | The plan: architecture, phases, decisions, open questions, checklist | The place for *how* and *how far*: status, progress and next steps live here, nowhere else |
| `research/` | Background studies the plan cites | Reference |
| `tooling/` | The tooling and its entry script `tooling/up` (the Cargo workspace is `Cargo.toml` at the root) | The code |

`spec/README.md` explains the spec and indexes its areas. Before starting a task, read the parts of `spec/current/`
and `plan/` it touches; the plan says what is decided. `.claude/rules/` adds detail when matching files are touched.

## Instructions from the user

- Record each message that specifies or changes **what is built** (the language, its implementation and its native
  interfaces, and decisions that adjust either) in `spec/log/`, **verbatim** (typos kept) and **before acting on
  it**, and update the affected `spec/current/` rules in the same commit; Claude's reading of it goes to
  `spec/interpretations.md`, kept apart, only where it is a genuine interpretation. The log is what a later review
  checks the implementation against. Requests about carrying out the work (do a review, merge, show or test
  something) and questions are not recorded; a decision reached through them is, with the message that made it.
  Steps (formats: `.claude/rules/spec.md`):
  1. **Log it.** Get an id with `tooling/up spec-id L <latest L id in use>` and append the message to
     `spec/log/YYYY-MM.md` (this month's file, created if missing), **verbatim** (typos kept, nothing paraphrased):
     heading `### L<id> · <date> · <title> (<kind>)`, a `Context:` line when the message answers something, then the
     message in a `text` fence. Never edit an earlier entry.
  2. **Check authority** (below). If the message contradicts a current rule and it is not reasonably clear the user
     intends that specific change, mark the rule `[NEEDS USER: …]`, ask the user to confirm the ramifications are
     intentional, and record the answer as its own log entry before folding it in.
  3. **Update `spec/current/`.** Rewrite, add or remove the rules of every area the message affects, so each states
     what is true now; add the new L id to each changed rule's `Source:` line; remove superseded content. A new rule
     gets the next unused id of its area; an id is never reused.
  4. **Interpretation.** Only where Claude reads something into the message beyond its words, add a J row (id from
     `tooling/up spec-id J <latest J id in use>`) to `spec/interpretations.md` and cite it in the rule's `Source:`.
  5. **Plan.** Update `plan/` wherever the message changes a decision.
  6. **Check and commit.** Run `tooling/up spec-check` and fix what it reports; commit steps 1–5 together as
     `spec: <what changed>`.
- When an instruction changes a decision, update `plan/` to match.
- **Authority.** The log wins over everything derived from it (`spec/current/`, interpretations, plan, code): where
  `spec/current/` disagrees with the log, that is a spec bug, fixed from the log. Within the log a later explicit
  user specification overrides an earlier one, except:
  - a user confirmation of a large set of spec updates is logged but never overrides an explicit user specification;
  - an explicit user confirmation of one isolated conflict resolution overrides the specifications it resolves;
  - a new specification contradicting a current rule overrides it only when it is reasonably clear the user intends
    that specific change; otherwise it is a spec bug: mark it `[NEEDS USER: …]` and get the user's confirmation that
    the ramifications are intentional (that answer is then an isolated conflict resolution).
- Instructions about how Claude should work (process, conventions, Git, reviews) are not recorded in the spec; they
  go into CLAUDE.md or `.claude/rules/` only after the user has explicitly confirmed that change in its own message.
- Keep CLAUDE.md and `.claude/` to standing instructions: no status, progress, next steps or open decisions.

## Code

- **Rust whenever possible.** The compiler, runtime, tooling and tests are Rust. Other languages are used where Rust
  can't do the job: **C and C++** for the native side of the interface work (headers, test fixtures and harnesses
  that check Upshot against real C/C++ code); shell for entry scripts; YAML for CI. Keep each to the part that
  needs it.
- Libraries follow the same order: the Rust standard library, then crates maintained by the Rust project
  (`rust-lang`), then well-established third-party crates only where nothing above fits.
- Code passes `cargo fmt --check` and `cargo clippy -- -D warnings`. Tests are Rust tests (`cargo test`) and run
  through the entry script (`tooling/up test`); native interop is tested against real C/C++ compiled in the test.
- `unsafe` stays at the native boundary: each block has a `// SAFETY:` comment saying why it holds, and the safe
  API around it is what the rest of the code calls.
- Every language rule (syntax, semantics, type rules, the native mapping) is implemented once and has tests; other
  layers call it rather than re-implement it.
- Prefer plain-text, diff-friendly formats (Markdown, TSV, JSON, TOML). Generated files say so at the top and are
  regenerated, never hand-edited.
- The repository holds no secrets: never print or commit one.

## Working style

- **Significant work is done by subagents.** Hand substantial tasks (code across files, large doc updates, reviews)
  to subagents with a self-contained brief: goal, files they own, constraints, what "done" looks like. Run
  independent ones in parallel, each with its own files. The main session plans, coordinates, checks results and
  talks to the user; quick lookups and small edits it does itself.
- No clutter at the repository root: scratch work goes in the session scratchpad.
- **GitHub issues for bugs.** A bug noticed that exists on `main` gets its own issue, whether or not the current
  branch fixes it, with one exception: when the branch fixes it and it is related to an open issue the branch is
  working on, a note about this incidental fix goes on that issue instead. A bug that exists only on the current
  branch needs no issue: the PR records it. An issue says what happens, where (file:line), the expected behaviour,
  full repro steps when possible (a deterministic test when the failure is transient) and a suggested fix. Mention
  it to the user; an issue never holds secrets.
- **A PR closes the issues it fixes**: its description names each one with a closing keyword (`Fixes #60`), so the
  merge closes it.

## Code review

When asked for a "code review" (full) or a "targeted code review" (only the changes since the last review):

1. Rebase the branch on `main`.
2. Open a PR if none is open for the branch.
3. Launch subagent(s) to review the PR: the whole PR, or for a targeted review the changes since the last review.
   Besides the code reviewer(s), always launch a dedicated **spec reviewer** subagent that checks:
   - the implementation against `spec/current/`: the rules the change touches, plus any current rule it may break;
   - the affected `spec/current/` rules against the log entries and interpretations they cite and any later log
     entries on the same topic, flagging rules that misstate, omit or outlive their sources (*Authority*).
4. Assess the findings and fix the ones worth fixing, with subagents for any significant amount of work. A reviewing
   subagent (the spec reviewer included) only reviews: it never fixes its own findings; fixes are made by another
   subagent (or the main session for small ones).
5. Ask the user only what can't genuinely be worked out without them.
6. When done, check whether another rebase is needed, then recommend: ready to merge, or still needs a targeted or a
   full code review.

## Git

- `main` is the main branch.
- Commit after every meaningful change and push before ending a session: the cloud container is reclaimed.
- Commit messages start with the area: `spec:`, `plan:`, `tooling:`, `research:`, or the crate's name for code
  under it.
- Start each session by syncing with the default branch (`git fetch origin && git merge origin/main`); get session
  branches merged back promptly, by **squash merge**.
- **Never merge a PR without the user's explicit permission to merge it**, given in its own message for that PR. A
  review or a "ready to merge" recommendation is not permission.
- Once a branch has been merged, never add to it: further changes go on a **new branch from `main`**
  (`git fetch origin && git checkout -b <new-branch> origin/main`).
- Don't bother deleting temporary branches once pushed to GitHub (e.g. a subagent's branch after it is merged).
