# Plan

How Upshot is built and how far it has got. Builds on `spec/current/` (cited by rule id, e.g. LANG-3) and the
studies in `research/`.

Status: **setup** (2026-10-24). The repository, its Claude configuration and the spec tooling (`tooling/up`) are in
place; what Upshot is about is specified (L426464 to L427098); nothing of the language is built yet.

## Focus

The feature language and a prototype runtime in Rust (LANG-7, RUN-2); little to no optimisation-language work, so
the prototype runs every program under its default strategy (J426465). No other runtime is built (RUN-3).

## The shape of Upshot

Two parts (LANG-6): the **feature language**, in which a program states its data as maintainable structures (MODEL-1),
data sources and computed values (MODEL-2 to MODEL-4) and actions (ACT-1 to ACT-4) checked against validation
constraints (CONS-1, CONS-2), divided into data zones (ZONE-1) that interact only through each other's APIs of exposed
values and actions (API-1 to API-6, ZONE-2, ZONE-3), with metaprogramming (META-1); and the **optimisation metadata**,
written in the optimisation language (OPT-1 to OPT-7), which decides how that data is physically held and which
runtime runs each zone (RUN-1) without changing what the program computes. All data is versioned and, ideally, the
dataset reports its own state (VER-1, VER-2).

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
- **The prototype runtime** (RUN-2): in-memory only at first, or with a local store; how zones talk to each other
  through their APIs inside one process, and how a C/C++ program reaches a zone's API (LANG-3).
- **APIs and zones** (API-1 to API-3, ZONE-1, ZONE-2): whether an API can span zones or a zone has several APIs,
  and whether one zone's computed values may depend on another zone's exposed values (and how they stay current).
- **Configuration values and web backends** (API-4 to API-6, ZONE-3, LANG-8): the suggested structure is the user
  zone and shared zone of ZONE-3. Open:
  - *a simpler way to state configuration values* (L427098 asks for one). *Proposal:* an API with configuration
    values is a **parameterised API**: a function from its configuration values to an API instance, as a function
    from arguments to a result. Each session holds one instance; everything computed inside is computed per
    argument, and two sessions with the same user share one instance. Sharing, caching and dropping instances is
    memoisation, an optimisation concern (OPT-7). Prior art: Skip's parameterised *resources* (research 01 §6).
  - *several APIs for different audiences* (users, admins): either a zone declares several APIs and the harness
    picks one per session, or one API whose exposed values and actions are computed from the user's role
    (META-1). With parameterised APIs the two meet: the role is a configuration value, and the API's shape is
    computed from it.
- **Metaprogramming** (META-1): what can be computed (types, fields, collections, APIs, constraints?), when (once
  per program version, per session, or at any time), and how a computed structure interacts with migration
  (MIG-1), static checking and the native interface (LANG-3), which needs shapes known ahead of time.
- **Exception handling** (ACT-4): how a function declares its handler, and what a handler receives (the failure,
  the rejected constraint); the handler runs as a separate action (L427098).
- **The optimisation metadata** (OPT-1 to OPT-7; little to no work for now, LANG-7): its format, what it can
  express (storage, in memory and/or database, indexes, materialised computed values), the default strategy
  without it (J426465), and how usage metrics are collected for generating it. Which databases it targets first
  (local; no cloud, LANG-4).
- **Identity and lifetime of objects in computed collections** (MODEL-5 to MODEL-7): the syntax for an object's
  identity (a key expression per computed collection) and for choosing A, B or C. *Proposal:* split lifetime in
  two. Which source values an object sees after it leaves and re-enters (A, B, C) changes results, so it is stated
  in the feature language (OPT-2 and J426465 keep results out of the metadata). How long the runtime keeps state
  that can no longer be observed (A's values for a key that can never come back, B's values once gone, old
  versions beyond what VER-1 keeps readable) is an optimisation concern. Related work: research 01.
- **Actions** (ACT-1 to ACT-3): provisional (J426468); concurrency between actions. *Within an atomic action*: an
  action that updates sources and then needs computed values that depend on those updates, to continue. Redoing
  the computations by hand in the action works in theory but is not practical (L427008). Candidates: the action
  reads a tentative view in which its own changes so far are applied and propagated (as databases let a
  transaction read its own writes), committed or discarded as a whole with the constraint check (CONS-2) at the
  end.
- **Converting between source and computed** (MODEL-9): which computation a source converted to a computed value
  gets (one declared in the program, or one the action supplies), and whether the program declares which values
  may be converted.
- **Validation constraints** (CONS-1, CONS-2): syntax; whether a constraint can be scoped to an object or
  collection; what happens when a program change (a new computation function) makes existing data violate a
  constraint. Tar Pit (research 01 §1, its §10.3.3) suggests treating the program change like a change to data
  and checking it, or refusing to run the new program until the data is repaired; this ties into migration.
- **Migration** (MIG-1): how data follows a program change: renames, reshaped structures, a source becoming
  computed or the reverse, new constraints; how optimisation metadata that names program elements survives
  renames; how versioned history reads across program versions. Tar Pit says almost nothing here: only that
  changes to the essential state's specification may force changes to the logic and the hints, never the
  reverse. Related work to study: Cambria's schema-version lenses (research 01 §4), Datomic's accrete-only schema
  and Dolt's schema diffs (§9).
- **Observers and history** (VER-1 to VER-3): observers are not yet defined (L427008). Questions:
  - is a value's version history read directly, or as an explicit list through observables (VER-3)?
  - are observer results automatic computed values, or components that trigger actions producing their own,
    separate change sets?
  - *long propagations*: long propagations may call for **propagation status properties** (values reporting how
    far a change has propagated, L427008 as corrected by L427057), and as these change, one source change can
    produce several successive versions of the same value elsewhere. Whether those intermediate versions are
    visible, and how they relate to self-inspection (VER-2), is unresolved. Timely dataflow's frontiers (research
    01 §9) are one model: a version is visible only once everything before it is complete.
  - what a version is (per value, per dataset, per action), and how long history is kept.
