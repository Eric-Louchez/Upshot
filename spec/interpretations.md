# Interpretations

Claude's interpretations of the user's messages (the log, `spec/log/`), kept apart from them; the user may
overrule them. Each row is `| J<id> | <sources> | <interpretation> |`; the sources column names the ids the row
reads. This file is append-only: rows are never renumbered, and new rows take their id from `tooling/up spec-id J`
(see [`README.md`](README.md)).

| # | Source | Interpretation |
|---|---|---|
| J426444 | L426443 "targets native interface integration. (C/C++)" | Upshot works with C and C++ code through their native interfaces (the C ABI and C/C++ headers), so programs and libraries written in C or C++ can use it and be used by it; the exact mechanism (generated bindings, headers, a runtime library) is a plan decision still to make. |
