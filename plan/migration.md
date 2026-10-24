# Migration — design study

How existing data, its history and its users follow a change to the program (MIG-1 to MIG-7). This is a study; the
decisions taken since are summarised below and stated in `spec/current/migration.md`. Everything else marked
**Proposal** is a proposal; where the spec decides something, the rule is cited.

Sources: L427008 (point 2: "an integrated migration story"), L427115 (persistent ids, under consideration, see
`README.md`), L427237 (the kinds of change to cover); research 01 §1 (Tar Pit), §4 (lenses, Cambria), §9 (versioned
data), §13 (identity and lifetime in computed collections).

## Decided since this study (L427445)

L427445 settles the main design (MIG-3 to MIG-7, VER-4); the sections below are read in its light:
- **Parallel program versions** (MIG-3): a snapshot of the whole program is kept for each supported version.
- **One sequence of transactions** (MIG-6, L427505): the migration is one atomic transaction among the actions;
  actions before it apply to the old schema, actions after it to the new. Upgrades are hot because the migration is
  a single transaction, not a stop. This answers §5 question 1 (one atomic step, not online or lazy) and most of
  question 6: an action is never split across versions (P3). Still open: what happens to a client or session built
  against the old version's API that submits an action after the migration.
- **The migration transform** (MIG-4) goes from the old version's data zones to the new version's sources. It is a
  metaprogramming operation (META-1), or a built-in one of the same kind. §3.1's classification still says which
  parts of it Upshot can supply automatically.
- **Automatic by matching structure** (MIG-5): a new source whose matching old value (source *or computed*) has the
  same format, type and structure is migrated with nothing declared. This widens §4's Automatic class: a value that
  was computed and becomes a source keeps its content.
- **A migration is an action** (MIG-6, §3.2): atomic, its own transaction version, checked against the new
  constraints; this answers §5 question 4 as P2 (a migration whose result breaks a constraint is rejected, and the
  old version keeps running).
- **History is never rewritten** (MIG-7, VER-4): old versions stay as they were, values may stop existing, new values
  start without history. This answers §5 question 3 (a past computed value is what was computed then) and most of
  question 2 (history spans program changes, P4).

Still open from §5: whether the transform is a lens by default (question 5), cut-over of zones on different runtimes
(7), accrete-only APIs (8), declaring convertible values (9), persistent ids (10), and when an old version stops
being served.

## 1 Framing

### What a migration has to carry

- **Source data** (MODEL-2). The only data that cannot be rebuilt: everything else is defined from it. This includes
  the source values carried by objects of computed collections (MODEL-5), keyed by the objects' declared identity
  (MODEL-6) and kept according to their lifetime A/B/C (MODEL-7).
- **Computed data.** Logically never migrated: it is redefined by the new program and recomputed. Whatever the
  runtime has stored is a cache under the optimisation metadata (OPT-2). Two exceptions make computed data matter:
  - a **bidirectional** value whose persisted form is the value, not its sources (MODEL-4, OPT-3): physically it *is*
    the authoritative data, so a change to its two-way function changes what the stored bytes mean;
  - **history** of computed values (VER-3): whether a past computed value is what was computed then, or what the new
    function computes from the past sources.
- **History** (VER-1, VER-3): all earlier versions, written under earlier programs.
- **Dependents**: other zones (they see this zone only through its APIs, ZONE-2), external API clients and the
  harness that supplies configuration values (API-4, LANG-8), every set of optimisation metadata (OPT-6), and
  anything in flight (queued actions, open sessions of an API).

**Proposal P1 — migration is logical.** A migration is defined on the logical dataset (source values, including
identity-keyed ones), never on the physical layout. The physical side (re-layout, re-indexing, which form of a
bidirectional value is stored) follows automatically from the new program plus its metadata, and is checked against
the from-scratch semantics like any other metadata choice (research 01 §14). This keeps migration in the feature
language and the optimisation metadata out of it (OPT-5).

### Kinds of handling

Used in every table below.

| Code | Meaning |
|---|---|
| **Auto** | Upshot derives the new dataset from the old with nothing declared |
| **Init** | Auto once the programmer gives an initial value (a constant, or an expression over the old dataset) |
| **Decl** | Needs a declared migration: a function from old data to new data |
| **Choice** | Needs a decision among a few policies (drop or keep, reject or repair, …); often a one-word declaration |
| **Break** | Orthogonal flag: breaks a dependent (API client, other zone, harness, metadata) unless a compatibility path is declared |

Reversibility: **Inv** (an inverse exists, e.g. a lens: no information lost), **Hist** (lossy, but the pre-migration
version stays readable in history, VER-1, if history crosses program versions, §3.6), **Loss** (lossy even with
history, e.g. history is truncated at the migration).

### What persistent ids do, in general

With L427115's ids, the old and new programs can be **diffed exactly**: every element of the new program is either
an old element (same id, possibly renamed, moved, retyped or redefined) or new, and every old element absent from the
new program was removed. Without ids a rename is indistinguishable from remove + add, and only heuristics or a
declaration tell them apart. Ids therefore turn rename/move into **Auto** and make every other change *detectable and
classifiable*; they do not say what the new data should be when the meaning of an element changes. They are the
same device as protobuf field numbers (§3.5): an id is never reused, and a removed id is retired.

## 2 Taxonomy

### 2.1 Sources and fields

| Change | Existing source data | Kind | Rev. | Ids |
|---|---|---|---|---|
| Add a source value / field | none yet; existing objects need a value | Init (Auto if optional or an empty collection) | Inv | new id |
| Remove a source | dropped from the current version | Choice (confirm the loss) | Hist | id retired |
| Rename | unchanged | Auto with ids; Decl (`was old`) without | Inv | solves it |
| Move (to another object, collection) | re-homed; needs a path old→new | Auto if the path is 1:1 (ids); Decl otherwise | Inv/Hist | keeps identity |
| Widen type (Int→Float, add an enum case) | converted losslessly | Auto | Inv | — |
| Narrow / change type | some values do not fit | Decl (+ Choice for misfits) | Hist | — |
| Split one value into several | each part computed from the old value | Decl | Inv if the split is lossless | parts are new ids |
| Merge several into one | combined | Decl | Hist usually | — |
| Nest / un-nest (group fields into a sub-object, or flatten) | restructured | Auto (pure restructuring) | Inv | keeps field ids |
| Single value → collection | old value becomes one element | Auto | Inv | — |
| Collection → single value | which element survives? | Decl / Choice | Hist | — |
| Reference ↔ embedded | embed: copy (shared targets duplicated); reference: extract and dedupe | Decl (identity of extracted objects) | Hist | — |
| Change a default value | none for existing data; new objects only | Auto | Inv | — |

Common effects across the table:
- **Computed data** over a changed source is recomputed (Auto). Computations whose definitions refer to the old form
  are program errors the compiler reports, not migrations.
- **History**: old versions keep the old shape (§3.6).
- **Constraints**: every constraint is re-checked on the migrated data (§3.2); a narrowing can make one false.
- **Dependents**: an exposed source that is removed, retyped or narrowed is **Break** for API clients (§2.6).
- *Moving* a source between collections (e.g. from each container to the crate) changes its cardinality: one value
  per container becomes one per crate, so it is in fact a merge (Decl) or a split.

### 2.2 Computed values and computed collections

| Change | Effect | Kind | Rev. | Ids |
|---|---|---|---|---|
| Add a computed value | computed from existing data | Auto | Inv | new id |
| Remove a computed value | nothing to carry (cache dropped) | Auto; Break if exposed | Inv | id retired |
| Rename / move | none | Auto with ids | Inv | solves it |
| Change its computation function | recomputed | Auto (semantic change: Choice for history, §3.6) | Inv | — |
| Change a function a constraint uses | constraint may now fail on existing data | see §2.4 | — | — |
| Change the function of a bidirectional value persisted as the value (OPT-3) | stored data must be re-read through the **old** `put` into sources, then re-derived with the new function | Auto (physical, via the old lens) | Inv | — |
| Change the identity key of a computed collection | objects' source values must be re-keyed | Decl (old key → new key), or Choice (drop unmatched) | Hist | keeps the collection, not the objects |
| Change a function the identity key depends on | same as a key change, without the key expression changing | Decl / Choice (detected by dependency analysis) | Hist | — |
| Change what the collection contains (filter, count) | objects leave or enter; their source values follow their lifetime (MODEL-7) | Auto under the declared lifetime | per lifetime | — |
| Lifetime A → B | stored values of objects currently absent are dropped | Choice (confirm loss) | Hist | — |
| Lifetime B → A | nothing to drop; values are kept from now on | Auto | Inv | — |
| Lifetime C → A or B | current objects gain an identity: key the current values by the new key | Auto if the key is unique; Decl otherwise | Inv | — |
| Lifetime A or B → C | identity dropped; next recompute yields fresh objects with initial values | Choice (confirm loss) | Hist | — |
| Add source values to a computed collection's objects | each existing object needs one | Init (expression may use the object) | Inv | new id |

Notes:
- The identity key is what makes a computed collection's source values data rather than cache. Changing it is the
  only change to a computed value that touches source data; Upshot can detect it (the key, or any function the key
  depends on, changed) and require a declaration. **Proposal:** the declared form is a function from the old key to
  the new one, evaluated on the stored entries, with a Choice for entries that map to no current object or collide.
- Changing an ordinary computation function changes **meaning** without changing data: an API client reading
  `total` gets different numbers. It is not a data migration but can be a semantic Break (§2.6).

### 2.3 Source ↔ computed, bidirectional ↔ one-way (program level)

MODEL-9 lets an **action** convert one value at run time; the program is unchanged, so the value's declaration must
already allow both states. A **program change** converts the declaration, for all instances at once.

| Change | Existing data | Kind | Rev. |
|---|---|---|---|
| Computed → source | the last computed value of each instance becomes its initial source value (MODEL-9 applied to all) | Auto | Inv (the old function is in history) |
| Source → computed | stored values are replaced by the function's results; where they differ, data is lost | Choice: discard differences, reject the migration if any differ, or Decl: keep differences as new override sources | Hist |
| Source → bidirectional computed over new sources | old values are `put` through the lens into the new sources | Auto if the lens is total | Inv |
| One-way → bidirectional | no data change; the value becomes writable (MODEL-8) | Auto | Inv |
| Bidirectional → one-way | no logical change; if the value was the persisted form (OPT-3) it is converted to sources via the old lens; read/write exposures become impossible (API-3) | Auto; Break for writers | Inv |

The "keep differences as overrides" option is the Tar Pit's "changeable derived data without an inverse is really
input" (L427008 point 3): the converted value becomes `computed default unless a source override is present`.
**Proposal:** offer this as a standard pattern in the language, since it is also what a MODEL-9 run-time conversion
needs: a declaration that can hold either a computed or a source state per instance.

### 2.4 Constraints

| Change | Existing data | Kind | Rev. |
|---|---|---|---|
| Add or tighten a constraint | may violate it | Auto if the data satisfies it; otherwise Choice: reject the migration, or Decl: a repair | Inv (constraint) / Hist (repair) |
| Change a computed value or function a constraint depends on | as tightening | as above (detected by dependency analysis) | — |
| Loosen or remove a constraint | always satisfied | Auto | Inv; reinstating it is a tightening |

CONS-1 says a constraint must always hold, so "grandfathering" old data that violates a new constraint is not
available as a policy: it would contradict CONS-1. A programmer who wants it writes the exemption into the
constraint (e.g. over a source flag set by the migration). **Proposal P2:** a migration is checked like an action
(CONS-2): if the migrated dataset breaks any constraint, the migration is rejected as a whole and the program change
is not deployed. Loosening a constraint never touches data but can change behaviour: actions that used to be
rejected now succeed, so clients that relied on the rejection see a semantic change.

### 2.5 Actions

Changing an action's body has no effect on stored data (L427237: "should not be much trouble"). What remains:

| Change | Effect | Kind |
|---|---|---|
| Change an action's body or handler (ACT-4) | none on data; new behaviour from the switch-over | Auto |
| Change an exposed action's parameters or result | callers break | Break unless a shim (an old-signature action calling the new one) is declared |
| Remove or rename an exposed action | callers break (rename: not with id-bound clients) | Break |
| Actions queued or in flight at the switch-over | validated and written against the old program | Choice (below) |
| History naming the action that produced a version (if provenance is kept) | refers to an old action | Auto with ids |

**Proposal P3 — an action runs entirely under one program version** (decided in substance by MIG-6). At the switch-over, in-flight actions either
finish under the old program before the migration runs (drain), or are rejected and reported as failures (ACT-4)
and retried by their caller. Queued actions not yet started are run under the new program, which is a Break for
their signature if it changed. Draining is simplest and matches ACT-3's atomicity; it requires the migration to run
between two versions of the dataset, never inside an action.

### 2.6 APIs

| Change | Existing clients / other zones | Kind |
|---|---|---|
| Add an exposed value or action | unaffected | Auto |
| Remove one | break | Break |
| Rename or move to another API | break for name-bound clients; not for id-bound ones (L427115) | Auto with ids, else Break |
| Change an exposed value's type or shape | break | Break unless a lens keeps the old shape (§3.4) |
| Change an exposed value's definition (same type) | structurally compatible, semantically different | Choice (programmer declares it compatible or not) |
| Read/write → read-only | writers break | Break |
| Read-only → read/write (MODEL-8 permitting) | unaffected | Auto |
| Add a configuration value (API-4) | the harness must supply it: every use breaks | Break (or Init: a default supplied by the runtime) |
| Remove a configuration value | harness supplies an unused value | Auto |
| Change a configuration value's type | harness breaks | Break |
| Change an API whose shape is computed (META-1) | compatibility must hold for every shape it can compute | Choice / Break; not decidable statically in general |

Configuration values are not persisted (API-4), so a change to them migrates no data; the parallel computed values
per configuration value (API-6) are recomputed. Sessions open at the switch-over hold an API shape fixed for the
session (META-1). **Proposal:** a session is bound to one program version, like an action (P3): it ends at the
switch-over, or it stays on the old API through a compatibility lens until it ends.

### 2.7 Zones

| Change | Effect | Kind |
|---|---|---|
| Add a zone | none | Auto |
| Remove a zone | its data dropped; zones using its API break | Choice + Break |
| Rename a zone | none with ids | Auto with ids |
| Move data from one zone to another | each datum belongs to exactly one zone (ZONE-2): the move is atomic across both zones, possibly two runtimes (RUN-1); direct references inside the old zone to the moved data become API uses, so the move forces program changes on both sides | Decl (which data, which APIs) |
| Split a zone | as moving part of its data to a new zone, plus new APIs between them | Decl |
| Merge two zones | API uses between them may become direct references; Auto for the data | Auto + program change |

Zones on different runtimes cannot always switch program versions at the same instant. Either a program change is
deployed to all zones of a program atomically (a distributed cut-over), or each zone's API must stay compatible
across the window in which the other zone runs the old program (§3.4, §3.5). Moving data between runtimes is also
where the history of the moved data has to go somewhere: with the data (the target zone holds history written
under another zone), or left behind.

### 2.8 Metaprogramming

Structure computed from values (META-1) blurs the line between a program change and a data change: changing a
role can change which values an API exposes, at run time, with no program change.
- Computed structure over computed values only: recomputed, nothing to migrate (Auto).
- Computed structure that carries **source** values (fields that exist only for some computed shape) has the same
  problem as computed collections (§2.2): when the shape recomputes, the source values need an identity and a
  lifetime. **Proposal:** treat it with MODEL-6 and MODEL-7, not as migration: source values of computed structure
  are keyed by a declared identity and follow a declared lifetime, so run-time shape changes are ordinary
  recomputation and only a change to the *meta-function* or the key is a program migration (as §2.2).
- A computed API shape: §2.6 (last row).

### 2.9 Optimisation metadata

Metadata never changes results (OPT-2), so a metadata change, or a program change that only affects the metadata,
should be a no-op logically. It still needs a **physical migration** (re-layout, rebuild indexes, flip the
persisted form of a bidirectional value, move a zone to another runtime, RUN-1), which is Auto but can be slow. What
can still go wrong:
- a hint naming an element that was **renamed or moved**: fixed by ids (L427115), otherwise silently stale;
- a hint naming an element that was **removed**: **Proposal:** ignored with a warning; since metadata is optional
  (OPT-1), a dangling hint never stops a program;
- a hint that no longer **fits** the element (an index on a value that became a collection, a stored layout for a
  changed type): ignored with a warning, as above;
- a lens that is not well-behaved: flipping the persisted form (OPT-3) through `put`/`get` changes data if the
  round-trip laws do not hold; checking the laws (research 01 §4) is what makes the flip safe;
- non-determinism: identities that depend on evaluation order (research 01 §13 point 4) would make results differ
  after a re-layout;
- a physical migration interrupted half-way: it must be atomic or resumable, and the logical dataset readable
  throughout;
- several metadata sets (OPT-6), some hand-written, some generated (OPT-4): each needs updating; generated ones
  are regenerated.

### 2.10 Changes to the persisted, authoritative form

If L427115 is adopted, the program itself is persisted in an authoritative language. That form changes too:
- **the Upshot language evolves** (new constructs, changed semantics): every stored program, and data written
  under it, carries the language version, and the tooling upgrades the authoritative form (Auto when purely
  syntactic, a migration like any other when semantics change);
- **merging branches** of a program: two branches adding the "same" element give it two ids; the merge needs a
  decision (keep both, or declare one id an alias of the other), and the data written on each branch, if any, needs
  the same;
- **undoing a change** in source control (reverting a commit) goes back to an earlier program, but not to earlier
  data: the data must migrate forward to the reverted program (§3.6);
- the runtime's own **storage format** is physical and follows the metadata (§2.9).

## 3 Mechanisms

### 3.1 Program diff and classification

**Proposal:** the tooling diffs the old and new programs (exactly with ids, by name otherwise), classifies every
element change with the tables of §2, applies the Auto ones, and refuses to deploy until every Init, Decl and Choice
is declared and every Break is either accepted or covered by a compatibility path. The classification is itself a
language rule, so per CLAUDE.md it is implemented once and tested.

### 3.2 A migration as an action

**Proposal:** a declared migration is a special action that reads the **old** dataset (under the old program) and
writes the **new** dataset's sources (under the new program):
- it is atomic (ACT-3): all of it or nothing;
- the new constraints are checked on its result (CONS-2); a failure rejects the program change, and an exception
  handler (ACT-4) can report it or try a fallback;
- it produces one new data version, whose predecessor is the last version under the old program (§3.6);
- it is usually written **per element**, next to the declaration it migrates (`oranges: Int, was crate.count * 2`),
  with a whole-dataset form for restructurings (split a zone, re-key a collection).

Two practical limits: a large dataset may not migrate in one transaction (online, lazy, per-object migration on
first read is the usual answer, but constraints over the whole dataset still need a full evaluation), and a
migration across zones on different runtimes needs a distributed commit.

### 3.3 Expand / contract, natively

Expand/contract (parallel change) adds the new form beside the old, keeps both in sync, moves the clients, then
removes the old. Upshot's bidirectional values make the "keep in sync" step part of the language:
1. **Expand:** declare the new form as the source; redeclare the old form as a bidirectional computed value over it
   (MODEL-4). Old API exposures stay read/write (API-3), old clients and other zones keep working.
2. **Migrate clients** to the new exposures at their own pace.
3. **Contract:** remove the old computed value and its exposures (Auto; Break only for clients that did not move).

This works for every change of §2.1 that has a lens (rename, nest, single → collection, split with inverse). For
lossy changes the old form can only be one-way, so old clients keep reading but cannot write.

### 3.4 Lenses between program versions (Cambria)

Cambria (research 01 §4) translates data between schema versions with a small set of lens operations: add, remove,
rename, hoist and plunge (move a field out of or into a sub-object), wrap and head (single value ↔ collection),
convert (map values both ways). **Proposal:** a declared migration that is a lens, not just a function, buys three
things at once: old clients keep working through `put` (an old API as a lens over the new data), old history is
readable under the new program through `get` (§3.6), and the migration is reversible (Inv). A plain function gives
only the forward direction. The lens laws (GetPut, PutGet) are checked as for any bidirectional value.

### 3.5 Accrete-only

Datomic and protobuf avoid most migrations by never changing the meaning of a name: elements are only added; a
removed element's name or number is retired, never reused. L427115's ids are exactly protobuf's field numbers.
**Proposal:** offer accrete-only as an opt-in discipline for an API (not for the whole program): an API declared
*stable* may only grow, so other zones and external clients never break; anything else goes through a new API
version, kept as a second API of the zone (API-1) computed over the same data (§3.3).

### 3.6 Versioning across program versions

**Proposal P4:** a dataset version is a pair (program version, data version). Every version records the program it
was written under; a migration is a version whose predecessor was written under the previous program. Then:
- reading history (VER-3) is defined: a past version is read **as it was**, under its own program (the old program
  is kept, so ids and old shapes stay meaningful), or **translated** to the current program through the migration
  chain, lazily, when the migrations are lenses or functions;
- a past computed value is, by default, what the program in force then computed; recomputing the past under the new
  function is a separate query ("as if"), which resembles bitemporality's two time axes (research 01 §9) with
  program time in place of valid time;
- rollback is either a forward migration to the reverted program (keeps data written since) or a reset to the
  pre-migration version (loses it); the second is only possible if history crosses the migration;
- retention (VER-1, open) decides whether **Hist** reversibility exists at all.

### 3.7 Compatibility rules for zones and APIs

**Proposal:**
- A program change is *compatible* for an API if every operation an old client can perform still has a meaning:
  additions only, or every changed element covered by a lens (§3.4) or kept by expand/contract (§3.3).
- Zones of one program on one runtime switch together, atomically. Zones on different runtimes must be compatible
  across the switch-over window, or the change is deployed with a coordinated cut-over.
- External clients bind to ids where they can (generated client code), so renames and moves never break them.

## 4 Proposed classification

**Proposal:** every program change is one of:
- **Automatic**: no declaration; nothing lost. Adding computed values, optional sources, zones, API elements;
  renames and moves (with ids); nest/un-nest, single → collection; changing computation functions not used by
  identity keys or constraints; computed → source; one-way → bidirectional; loosening or removing a constraint;
  action bodies; all metadata changes.
- **Declared**: Upshot needs an initial value, a migration function or a lens. Adding a required source; type
  narrowing, split, merge, collection → single, reference ↔ embedded; identity-key changes; source → computed with
  differences kept; moving data between zones; a new constraint that existing data violates (repair).
- **Destructive** (a declared confirmation that data is lost; recoverable only through history): removing a source
  or a zone; lifetime A → B or → C; source → computed discarding differences.
- **Breaking** (orthogonal, for dependents): removing, retyping or making read-only an exposed element; changing an
  exposed action's signature; adding a configuration value; removing an API or zone used by others. Deployed only
  with a compatibility path (lens, expand/contract, a kept API version) or an explicit acceptance.

## 5 Open questions for the user

1. **When does a migration run?** At deployment, as one atomic step that blocks the zone (simple, exact constraint
   checks), or online and lazily per object (no downtime, constraints harder)? Or both, chosen by the metadata?
2. **Is the program version part of the data version** (P4), so history spans program changes and old versions
   are read under their own program?
3. **History of computed values** after a function changes: what was computed then, or recomputed under the new
   function?
4. **New constraints and existing data**: is rejecting the program change (P2) right, or should a constraint be
   able to apply only from its introduction on (which would soften CONS-1's "must always hold")?
5. **Function or lens**: should a declared migration be a lens by default (reversible, old clients and history keep
   working), with a one-way function allowed only when the change is lossy?
6. **In-flight actions and open sessions** at the switch-over: drain, reject or translate (P3)?
7. **Zones on different runtimes**: coordinated cut-over, or per-zone deployment with API compatibility windows?
8. **Stable APIs**: is an accrete-only discipline (§3.5) wanted for APIs used by other zones or external clients?
9. **Program-level vs run-time conversion**: should a value that actions may convert (MODEL-9) declare that
   possibility, e.g. as "computed default with source override" (§2.3)?
10. **Persistent ids** (L427115): does the plan adopt them? Most of the Auto column for renames and moves, and stale
    metadata hints, depend on it.
