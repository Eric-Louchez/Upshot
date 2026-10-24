# Plan

How Upshot is built and how far it has got. Builds on `spec/current/` (cited by rule id, e.g. LANG-3) and the
studies in `research/`.

Status: **setup** (2026-10-24). The repository, its Claude configuration and the spec tooling (`tooling/up`) are in
place; what Upshot is about is specified (L426464); nothing of the language is built yet.

## The shape of Upshot

Two parts (LANG-6): the **feature language**, in which a program states its data as maintainable structures
(MODEL-1), data sources and computed values (MODEL-2 to MODEL-4) and actions (ACT-1, ACT-2); and the
**optimisation metadata** (OPT-1 to OPT-4), which decides how that data is physically held without changing what
the program computes. All data is versioned and, ideally, the dataset reports its own state (VER-1, VER-2).

## Open questions

- **Surface syntax and embedding.** Is the feature language a standalone language with its own compiler, a Rust
  DSL (macros), or a schema-like description with expressions? How does a C or C++ program define and use a
  dataset (LANG-3)?
- **The native interface mechanism** (LANG-3, J426444): generated C/C++ headers and a runtime library linked into
  C/C++ programs, or Upshot code calling C/C++ libraries, and which C++ features it must reach beyond the C ABI.
- **Computed values** (MODEL-3): the expression language for arbitrary computed values and collections, and how
  computed data stays consistent: incremental recomputation, invalidation, eager or lazy.
- **Bidirectional values** (MODEL-4): how a two-way path is declared (paired functions, invertible operators) and
  what happens when a write cannot be carried back unambiguously.
- **The optimisation metadata** (OPT-1 to OPT-4): its format, what it can express (storage, in memory and/or
  database, indexes, materialised computed values), the default strategy without it (J426465), and how usage
  metrics are collected for generating it. Which databases it targets first (local; no cloud, LANG-4).
- **Actions** (ACT-1, ACT-2): provisional (J426468); transactions, concurrency, and validation of inputs.
- **Versioning** (VER-1, VER-2): what a version is (per value, per dataset, per action), how long history is kept,
  and how self-inspection appears in the language.
