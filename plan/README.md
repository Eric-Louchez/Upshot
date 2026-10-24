# Plan

How Upshot is built and how far it has got. Builds on `spec/current/` (cited by rule id, e.g. LANG-3) and the
studies in `research/`.

Status: **setup** (2026-10-24). The repository, its Claude configuration and the spec tooling (`tooling/up`) are in
place; nothing of the language is built yet.

## Open questions

- **What "data-driven aspects of development" covers** (LANG-1): the kinds of programs and data Upshot is for, and
  what it does that C, C++ or Rust alone do not. Needed before a language design.
- **The native interface mechanism** (LANG-3, J426444): how Upshot meets C and C++ (generated bindings, headers, a
  runtime library linked into C/C++ programs, or Upshot code calling C/C++ libraries), and which C++ features it
  must reach beyond the C ABI.
