# Plan

How Upshot is built and how far it has got. Builds on `spec/current/` (cited by rule id, e.g. LANG-3) and the
studies in `research/`.

Status: **setup** (2026-10-24). The repository, its Claude configuration and the spec tooling (`tooling/up`) are in
place; what Upshot is about is specified (L426464 to L427101; L427115 under consideration); nothing of the language
is built yet.

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

## Under consideration: an authoritative, id-based persisted form

L427115 (strongly considered, not yet decided): the programmer writes in a **user-facing language**, but the program
is persisted in source control in an **authoritative language** in which every externally visible variable has a
persistent, globally unique id (a GUID or OID), every reference uses the id, and the name is only a label. Renaming
is free, a function moved from one API to another keeps its id, and components not tightly connected to the program
(its optimisation metadata, programs that depend on its APIs, other zones) keep up through the ids.

What it would settle or ease:
- **Metadata in sync** (OPT-5): hints refer to ids, so renames and moves never break them.
- **Migration** (MIG-1, MIG-2): every kind of program change, not only renames and moves (L427237); the taxonomy
  and mechanisms are studied in [`migration.md`](migration.md).
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
