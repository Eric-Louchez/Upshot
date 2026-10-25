# Plan

How Upshot is built and how far it has got. Builds on `spec/current/` (cited by rule id, e.g. LANG-3) and the
studies in `research/`.

Status: **setup** (2026-10-24). The repository, its Claude configuration and the spec tooling (`tooling/up`) are in
place; what Upshot is about is specified (L426464 to L428317; L427115 under consideration); nothing of the language
is built yet.

## Focus

The feature language and a prototype runtime in Rust (LANG-7, RUN-2); little to no optimisation-language work, so
the prototype runs every program under its default strategy (J426465). No other runtime is built (RUN-3).

## The shape of Upshot

Two parts (LANG-6): the **feature language**, in which a program states its data as maintainable structures (MODEL-1),
data sources and computed values (MODEL-2 to MODEL-11) and actions (ACT-1 to ACT-4) checked against validation
constraints (CONS-1, CONS-2), divided into data zones (ZONE-1) that interact only through each other's APIs of exposed
values and actions (API-1 to API-6, ZONE-2, ZONE-3), with metaprogramming (META-1); and the **optimisation metadata**,
written in the optimisation language (OPT-1 to OPT-9), which decides how that data is physically held and which
runtime runs each zone (RUN-1) without changing what the program computes. All data is versioned and, ideally, the
dataset reports its own state (VER-1, VER-2).

## Under consideration: an authoritative, id-based persisted form

L427115 (strongly considered, not yet decided): the programmer writes in a **user-facing language**, but the program
is persisted in source control in an **authoritative language** in which every externally visible variable has a
persistent, globally unique id (a GUID or OID), every reference uses the id, and the name is only a label. Renaming
is free, a function moved from one API to another keeps its id, and components not tightly connected to the program
(its optimisation metadata, programs that depend on its APIs, other zones) keep up through the ids.

What it would settle or ease:
- **Metadata in sync** (OPT-5): hints refer to ids, so renames and moves never break them.
- **Migration** (MIG-1): renames and moves are no-ops for stored data; what is left to migrate is structural (a
  field removed, a type changed, a source becoming computed), and an id-based diff states exactly that.
- **Dependencies**: a dependency's refactoring reaches its users as id-preserving changes rather than breakage.

Questions it raises:
- *Who assigns ids, and how a rename is told from a delete plus an add.* Writing plain text in the user-facing
  language loses that intent. Options: an editor or language server that tracks identity (the way projectional
  editors do), explicit refactoring commands, or matching the new text to the old authoritative form on save
  (heuristics, an LLM, the programmer confirming ambiguous cases).
- *What gets an id*: "externally visible" variables (exposed values, actions, sources, computed values, types,
  fields, zones, APIs); whether internal names need ids too.
- *Readability in source control*: an authoritative text that still diffs and reviews well (e.g. each id next to
  its current label), and merge conflicts when two branches create or rename the same element.
- *Copy and paste*: whether a copied element keeps its id or gets a fresh one.
- *Interaction with metaprogramming* (META-1): ids for structure computed at run time.

Prior art to study: Unison (definitions identified by content hash, names stored apart as labels; research 01 §10),
JetBrains MPS (projectional editing, references by persistent node id), Darklang (an id on every expression),
Protocol Buffers field numbers and Cap'n Proto's 64-bit ids (persistent ids for schema elements, names free to
change), Datomic attributes (entities referred to by id).

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
- **APIs and zones** (API-1 to API-3, ZONE-1, ZONE-2): whether an API can span zones (a zone can have several, API-1),
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
- **Performance and relaxations** (OPT-8, OPT-9, LANG-9, REL-1 to REL-3): optimisations that Upshot's hard
  constraints rule out, recovered by relaxation annotations the developer chooses; which annotations exist and their
  syntax: [`performance.md`](performance.md).
- **The optimisation metadata** (OPT-1 to OPT-9; little to no work for now, LANG-7): its format, what it can
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
- **Actions and computation functions** (ACT-5, L428000): an action is like a computation function temporarily
  connected to the sources it modifies, but the direction is inverted: a computed value reads its sources and stays
  connected; an action writes its sources and is connected for one transaction. Ways to reconcile the two
  perspectives (proposals):
  - *an action is a computed value frozen into sources*: the action's results are computed values over its inputs
    and the current dataset, connected to the sources they target for the transaction, then converted to sources
    and disconnected at commit (MODEL-9). Writing and computing are then one mechanism, and an action can use every
    structure and annotation a computed value can;
  - *an action is a write through a lens*: it assigns new values to (possibly computed) targets, and the two-way
    paths (MODEL-4) carry them back to sources, so an action only ever "computes" its targets' new values;
  - *event sourcing*: the sequence of actions is the real source, and every data source is computed as a fold
    over the actions. L428060: this needs every version of every computed value (action n reads what action n−1
    wrote) and leaves cycles that resolve only dynamically; nor is it the inversion meant. *Claude's note:* the
    cycles disappear if versions are explicit in the semantics (action n reads version n−1 and defines version n,
    as Dedalus and Bloom do, research 01 §11), so this is at most a storage model, not a way of writing.
  The inversion meant (L428060) is about how functions are written: both the sequential and the per-target form
  are allowed for actions and computed values (ACT-6). Open: inside a sequential action, whether a later step sees
  a value an earlier step wrote (read-your-own-writes, see Actions above); the per-target form reads only the dataset
  before the action.
- **Cycles in computed dependencies** (L428060, decided by L428306: MODEL-10, MODEL-11): no fixed points and no
  run-time cycle within a transaction; a static cycle is a compile-time error unless explicitly annotated. Open: the
  annotation's syntax and scope (on each value of the cycle, or on the cycle as a whole: TBD, L428316); and how the
  compiler reports a static cycle, e.g. suggesting a rewrite that names the roles (`first = data`,
  `second = first + 1`): TBD. Data can be cyclic (L428316), so a computed value recursing over data that happens to
  be cyclic meets a run-time cycle: the action that triggers it fails and is rolled back (L428317, MODEL-10).
  Open: how the runtime detects a cycle that an action would create in a value nobody reads, without evaluating
  every computed value eagerly (e.g. by tracking which annotated cycles the changed data can reach).
- **Converting between source and computed** (MODEL-9): which computation a source converted to a computed value
  gets (one declared in the program, or one the action supplies), and whether the program declares which values
  may be converted.
- **Validation constraints** (CONS-1, CONS-2): syntax; whether a constraint can be scoped to an object or
  collection; what happens when a program change (a new computation function) makes existing data violate a
  constraint. Tar Pit (research 01 §1, its §10.3.3) suggests treating the program change like a change to data
  and checking it, or refusing to run the new program until the data is repaired; this ties into migration.
- **Migration** (MIG-1 to MIG-7): decided in outline (L427445): parallel program versions kept as snapshots, a
  migration transform from the old zones to the new sources, automatic where structure matches, run as one atomic
  transaction between actions (L427505), history never rewritten, hot upgrades. The taxonomy, mechanisms and
  remaining questions are in [`migration.md`](migration.md).
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
