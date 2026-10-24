# spec/

What is built, as the user asked for it.

| Path | What | Changes |
|---|---|---|
| [`log/`](log/) | The user's messages, **verbatim** (typos kept), one `### L<id> · <date> · <title> (<kind>)` entry each, with a Context line and the message in a `text` block. The authority; a later message overrides an earlier one. | Append-only: entries are never edited or renumbered. |
| [`interpretations.md`](interpretations.md) | Claude's interpretations, one `\| J<id> \| <sources> \| <text> \|` row each; the sources column names the ids the row reads. | Append-only. |
| [`current/`](current/) | The curated current specification per area, `current/<area>.md` (index below), whose `Source:` lines cite log and interpretation ids. | Edited in place to match the log and the interpretations. |

## Areas

| File | Ids | What it covers |
|---|---|---|
| [`current/language.md`](current/language.md) | LANG- | What Upshot is and why, its two parts (the feature language and the optimisation metadata), what it is implemented in, what it targets, what is out of scope and the current focus. |
| [`current/api.md`](current/api.md) | API- | APIs: exposed values (read-only or read/write), exposed actions, configuration values. |
| [`current/zones.md`](current/zones.md) | ZONE- | Data zones, how they interact, the suggested web backend zones. |
| [`current/runtime.md`](current/runtime.md) | RUN- | Runtimes: the assignment of zones to runtimes, the prototype runtime, later runtimes. |
| [`current/constraints.md`](current/constraints.md) | CONS- | Validation constraints and the rejection of actions that break them. |
| [`current/meta.md`](current/meta.md) | META- | Metaprogramming: structure and APIs computed from other values. |
| [`current/migration.md`](current/migration.md) | MIG- | How existing data follows changes to the program. |
| [`current/model.md`](current/model.md) | MODEL- | The data a program describes: data structures, data sources, computed and bidirectional values. |
| [`current/optimisation.md`](current/optimisation.md) | OPT- | The optimisation metadata: what it decides and where it comes from. |
| [`current/actions.md`](current/actions.md) | ACT- | Actions: how a program changes its data. |
| [`current/versioning.md`](current/versioning.md) | VER- | Versioned data and the dataset's self-inspection. |

## Log files

- `log/<yyyy>-<mm>.md`: the messages, in the file of the month (UTC) their id falls in, e.g. `log/2026-10.md`.
  Entries are in ascending id order in every file.

## Ids

Ids never change, since the plan and the code cite them. Ids are **date-based**: the minutes from
2026-01-01T00:00Z (UTC) to the time the id is made, e.g. L426443, or the latest id in use + 1 if that is higher;
an id that already exists for its letter is skipped. Make them with the tool, never by hand: give it the latest id
in use for that letter and how many ids are needed; it reads the clock itself and prints them one per line, each
free in the spec files and distinct from the others.

    tooling/up spec-id L 426443 3   # three message ids after L426443
    tooling/up spec-id J 426443     # one interpretation id
    tooling/up spec-check           # duplicates, malformed headings, order, dangling ids in sources; counts
