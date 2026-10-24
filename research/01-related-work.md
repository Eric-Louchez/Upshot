# 01 · Related work

Upshot separates what a program's data *is* (the feature language: rich structures, data sources, computed values,
bidirectional values, actions, APIs and zones) from how it is *held* (the optimisation metadata: stored or recomputed,
memory or database, indexes, runtime placement), with the rule that the metadata changes performance, never results
(LANG-5, LANG-6, OPT-1 to OPT-4). This study surveys work that shares parts of that idea, to see what Upshot can
borrow and where it is new. Each entry says what the work is, how it relates to Upshot, and where to read it. The
last two sections focus on the hardest open problem found so far, identity and lifetime of objects created by
computed collections, and list takeaways.

Researched 2026-10. Every work was verified against its paper, documentation or repository; every URL resolved
when this was written (a few university hosts only through search indexes).

## §1 Logical and physical data independence

**Codd, "A Relational Model of Data for Large Shared Data Banks" (CACM 1970).** The paper that founded the
relational model. Its stated central problem is *data independence*: users and application programs should not be
affected when the internal representation of the data changes. The relational view was the means; the goal was
exactly Upshot's split.
*Relation to Upshot:* Upshot is a renewed attempt at Codd's promise, extended from the database to the program's
in-memory data, and to structures (cycles, nested collections, collection references) that the relational model
normalises away. SQL achieved physical independence for indexes and access paths but not for materialisation
(materialised views are declared by hand and change the schema), nor for in-memory structures.
<https://web.eecs.umich.edu/~michjc/eecs584/Papers/codd_1970.pdf>

**ANSI/SPARC three-schema architecture (1975).** External, conceptual and internal schemas, with *physical* data
independence (conceptual schema immune to changes of storage structures and indexes) and *logical* data
independence (external views immune to conceptual changes). Date notes that few real systems achieve it fully.
*Relation:* Upshot's feature language plays the conceptual schema, the optimisation metadata the internal schema,
and APIs (API-1 to API-3) the external schemas. The vocabulary is useful for the plan.
<https://en.wikipedia.org/wiki/ANSI-SPARC_Architecture>

**Moseley & Marks, "Out of the Tar Pit" (2006).** Argues that state is the main source of accidental complexity and
proposes *Functional Relational Programming* (FRP), with a system split into: **essential state** (base relvars,
only data input by users), **essential logic** (derived relvars defined by relational algebra plus pure functions,
and integrity constraints), and **accidental state and control**, "a declarative specification of a set of
performance optimizations", i.e. *performance hints* that "can never affect the other specifications" (e.g. which
derived relvars to store, indexes). Feeders and observers connect to the outside world. It also observes that
mutable derived data "makes sense only where the function … used to derive the data has an inverse".
*Relation:* the closest prior statement of Upshot's design: data sources = essential state; computed values =
derived relvars; optimisation metadata = accidental state and control; bidirectional computed values = invertible
mutable derived data; observers ≈ exposed values, feeders ≈ actions. Differences: FRP is relations-only, while
Upshot keeps object-like structures with cycles and collection references (MODEL-1); FRP was never implemented as
described; it says nothing about zones, runtimes, versioning or metadata generated from metrics or LLMs. Upshot can
borrow the strict rule that hints never refer to each other or change logic, and FRP's argument for why derived
data should not be stored "in the ideal world".
<https://curtclifton.net/papers/MoseleyMarks06a.pdf>

## §2 Separating the algorithm from its schedule (analogies outside databases)

**Halide (SIGGRAPH 2012, PLDI 2013).** An image-processing DSL that separates the *algorithm* (what each pipeline
stage computes) from the *schedule* (tiling, fusion, storage vs recomputation, vectorisation, parallelism); the
schedule cannot change the output. Schedules are written by hand or found by search (later autoschedulers learn
them).
*Relation:* the clearest proof that "metadata changes performance, never results" works in practice, and that the
store-vs-recompute choice (Halide's `compute_at`/`store_at`) can be a schedule decision. Upshot's optimisation
language is a schedule for data rather than for loops; Halide's schedule-search and autoscheduler line is a model for
OPT-4 (generated metadata).
<https://commit.csail.mit.edu/papers/2013/halide-pldi13.pdf>

**Taichi (SIGGRAPH Asia 2019).** A language for computation over spatially sparse data structures in which the data
structure (hierarchy of dense, hashed, pointer or bitmasked levels) is declared separately from the kernels, which
are written as if against a dense array.
*Relation:* the same split applied to data layout: change the structure without touching the computation. A good
precedent for layout choices in the optimisation language.
<https://yuanming.taichi.graphics/publication/2019-taichi/>

**TACO, format abstraction (OOPSLA 2018).** The tensor algebra compiler describes each tensor storage format (dense,
CSR/CSF, COO, DIA, ELL, hash) by per-dimension capabilities, and generates code for any expression over any
combination of formats.
*Relation:* shows how to make physical formats pluggable behind one logical type via a capability interface. Upshot's
runtime could describe storage choices (vector, hash map, B-tree, database table) by capabilities in the same way.
<https://arxiv.org/abs/1804.10112>

**Legion and Regent (Stanford).** Data is organised in *logical regions* that can be partitioned (even with
overlap); tasks declare which regions they access with what privileges; a separate *mapper* decides which processor
runs each task and which memory holds each region instance. Mapping decisions "only impact performance and are
orthogonal to correctness". Regent is the language on top.
*Relation:* both halves of Upshot's metadata (where data lives, which runtime runs what) as a correctness-neutral
mapping interface. Legion's declared access privileges are analogous to zones interacting only through APIs.
<https://legion.stanford.edu/overview>

**Dyna (JHU).** A weighted generalisation of Datalog: a program is a set of equations over a possibly cyclic
computation graph, answering queries and propagating updates lazily or eagerly. The MAPL 2017 paper proposes a
solver that *learns* the execution strategy (laziness, memoisation, loop order, storage and indexing) by
reinforcement learning from the workload.
*Relation:* very close in spirit to OPT-4: the program fixes the values, and a separate, adaptively chosen strategy
decides what is memoised and how it is indexed. Dyna also treats aggregation (counts, sums) uniformly with rules.
<https://www.cs.jhu.edu/~jason/papers/vieira+al.mapl17.pdf>, <https://dyna.org/>

## §3 Incremental and self-adjusting computation

**Self-adjusting computation (Acar, CMU thesis 2005).** Programs are written as ordinary functional code over
*modifiable references*; the runtime records a dynamic dependence graph and, when inputs change, re-runs only the
affected parts (*change propagation*), with memoisation to reuse work.
*Relation:* the theory behind keeping computed values consistent (MODEL-2) without hand-written invalidation.
<https://www.csd.cmu.edu/sites/default/files/phd-thesis/CMU-CS-05-129.pdf>

**Jane Street Incremental (OCaml, 2015).** A production self-adjusting computation library: a dynamic dependency
graph of incremental values, used for trading UIs and spreadsheet-like calculations; only *observed* nodes are kept
up to date.
*Relation:* a pragmatic design (observers, cutoffs to stop propagation when a value is equal, bind for dynamic graphs)
that a Rust prototype runtime could follow for computed scalars.
<https://blog.janestreet.com/introducing-incremental/>

**Adapton (PLDI 2014) and Nominal Adapton (OOPSLA 2015).** Demand-driven incremental computation: thunks are
recomputed only when an observer demands them, and results can be reused across contexts. *Nominal* Adapton adds
first-class **names** that the programmer attaches to allocations and thunks, so that a recomputation can recognise
"the same" node even when its position in the computation changes; it is proved *from-scratch consistent*.
*Relation:* demand-driven vs eager is exactly a metadata choice Upshot wants (OPT-2). Names are the main prior work on
identity of things created *during* a recomputation (see §13). From-scratch consistency is the formal version of "the
metadata never changes results".
<https://www.cs.umd.edu/~mwh/papers/hammer13adapton.html>, <https://arxiv.org/abs/1503.07792>

**Salsa (Rust).** The incremental framework behind rust-analyzer and ty: memoised *tracked functions* over *inputs*,
with revisions, early cutoff ("backdating") and durability levels. It has three kinds of structs with explicit
identity rules: **inputs** (identity from creation, live until removed), **interned** structs (identity = field
values; equal fields give the same id), and **tracked** structs, created inside a tracked function, whose identity
is the creating query + a hash of its identity fields + a *disambiguator* counting duplicates within that query
execution. A tracked struct that the query does not re-create in a new revision is deleted.
*Relation:* the most directly reusable design for Upshot's Rust prototype, and an exact precedent for objects created
by computed collections, with the "dies with it" lifetime (§13). Salsa is a candidate dependency or reference
implementation for the prototype runtime (RUN-2).
<https://salsa-rs.github.io/salsa/overview.html>,
<https://github.com/salsa-rs/salsa/blob/master/book/src/plumbing/tracked_structs.md>

**Differential dataflow and Materialize.** Collections are multisets of `(data, time, diff)` updates; operators
(map, join, reduce, iterate) consume and produce differences, so incremental maintenance of recursive queries is
automatic. *Arrangements* are shared, multi-versioned indexes; timely dataflow's *frontiers* tell each operator which
timestamps are complete. Materialize builds an SQL database on it.
*Relation:* the computational core Upshot's computed collections need (MODEL-3), in Rust. Arrangements are a model
for "an index is metadata"; frontiers are a model for VER-2 self-inspection ("the result for version t is not
available yet").
<https://github.com/TimelyDataflow/differential-dataflow>,
<https://materialize.com/docs/overview/arrangements/>

**DBSP and Feldera (VLDB 2023 best paper).** A small algebra of streams (lift, delay, integrate, differentiate) in
which incremental view maintenance of any query is derived mechanically; SQL and Datalog with recursion and
aggregation are expressed in it. Feldera is the Rust engine built on it.
*Relation:* gives Upshot a principled way to derive the incremental form of each computed value from its definition,
rather than inventing an invalidation scheme per construct. Rust, MIT-licensed.
<https://www.vldb.org/pvldb/vol16/p1601-budiu.pdf>, <https://docs.feldera.com/literature/papers/>

**Noria (OSDI 2018).** A dataflow system for web applications that precomputes query results from a relational
schema and parameterised queries, with *partially stateful* operators whose state is filled on demand and can be
evicted.
*Relation:* "materialise some keys of a computed collection, compute the rest on demand" is a fine-grained metadata
choice Upshot could express; Noria shows it is implementable with eviction and upqueries.
<https://www.usenix.org/conference/osdi18/presentation/gjengset>

**Differential Datalog, DDlog (VMware, archived).** A Datalog dialect compiled to Rust on differential dataflow; the
programmer writes non-incremental rules and gets incremental maintenance, with C, Rust and Java bindings.
*Relation:* evidence that a declarative language compiled to Rust with a C interface works for incremental data
(LANG-2, LANG-3); also a warning about memory overhead and compile-time-only programs.
<https://github.com/vmware-archive/differential-datalog>

## §4 Bidirectional transformations, lenses and the view update problem

**View update problem; Bancilhon & Spyratos, "Update Semantics of Relational Views" (TODS 1981).** A view update is
translatable when it can be carried back to the base data leaving a chosen *complement* constant; complements are not
unique, so the translation is a policy choice.
*Relation:* the theory of MODEL-4 and ACT-2: writing to a bidirectional computed value is a view update. The
"constant complement" is a precise answer to "what happens when a write cannot be carried back unambiguously" (plan
open question): the definition must fix the complement.
<https://cs.uwaterloo.ca/~david/cs848/background/constant-complement.pdf>

**Lenses (Foster et al., POPL 2005) and Boomerang (POPL 2008).** A lens is a pair `get`/`put` obeying round-trip laws
(GetPut, PutGet); lenses are built from combinators, so every program is bidirectional by construction. Boomerang
adds *resourceful* (dictionary) lenses: chunks of the source are matched to the view by **keys**, so reordering the
view does not scramble hidden data.
*Relation:* lens combinators are the obvious way to declare two-way paths (MODEL-4) so that well-behavedness is
checked rather than hoped for. Resourceful lenses are prior work on identity: keys decide which hidden source data
goes with which view element after an edit, the same question as §13.
<https://www.cis.upenn.edu/~bcpierce/papers/harmony_bib.html>,
<https://www.cis.upenn.edu/~bcpierce/papers/boomerang.pdf>

**Relational lenses (PODS 2006) and Incremental Relational Lenses (ICFP 2018).** Lenses over relations (select,
project, join) whose well-behavedness is checked with functional dependencies; the 2018 work makes them incremental
(propagating deltas) and integrates them in the Links language.
*Relation:* bidirectional values over collections, not just scalars, and combined with incrementality, which is what
Upshot needs for a bidirectional computed collection.
<https://arxiv.org/abs/1807.01948>

**BX community.** The cross-disciplinary bidirectional transformations community (databases, PL, model-driven
engineering), with a wiki, a benchmark collection and a yearly workshop.
*Relation:* the place to find the terminology (well-behaved, very well-behaved, least change, symmetric lenses) for
specifying MODEL-4 precisely.
<http://bx-community.wikidot.com/>

**Cambria (Ink & Switch, 2021).** Edit lenses that translate data between schema versions on the fly in
local-first software.
*Relation:* lenses applied to schema evolution of versioned data; relevant if Upshot's versioning (VER-1) must read
data written under an older program.
<https://www.inkandswitch.com/cambria/>

## §5 Datalog and relational languages for applications

**Eve (2014–2018).** A language and environment by the Light Table team: a temporal logic language (inspired by
Dedalus/Bloom) over a relational datastore, where programs are blocks that search and commit facts, written as
literate documents. Shut down in 2018.
*Relation:* the most ambitious attempt to build whole applications on a declarative data core; its retrospective is a
caution about adoption when everything, including UI, is relational. Upshot keeps a familiar structure-based feature
language and targets integration with existing C/C++ instead.
<https://chris-granger.com/2016/07/21/two-years-of-eve/>, <https://futureofcoding.org/essays/eve>

**DataScript.** An immutable in-memory triple store (EAV with EAVT/AEVT/AVET indexes) with Datalog queries, rules and
a Pull API, for browser apps; databases are cheap persistent values.
*Relation:* shows that an entity–attribute model with references (including cycles) and a declarative query layer
fits in memory and is cheap; Upshot's MODEL-1 structures could be lowered to such a store as one physical option.
<https://github.com/tonsky/datascript>

**Logica (Google, 2021).** A Datalog-family language with aggregation that compiles to SQL (BigQuery, PostgreSQL,
SQLite), adding modules and testability to queries.
*Relation:* compiling a nicer logical language to an existing database engine is one route for Upshot's database
backends.
<https://opensource.googleblog.com/2021/04/logica-organizing-your-data-queries.html>

**Soufflé.** A Datalog engine that synthesises C++ from programs (used for large-scale program analysis), with
records, ADTs and user-defined functors.
*Relation:* proof that Datalog compiles to fast native code with a C++ interface (LANG-3); a reference for
specialising data structures per relation, a decision Soufflé takes automatically.
<https://souffle-lang.github.io/>

**CozoDB.** An embeddable transactional relational database in Rust with Datalog queries, graph algorithms, opt-in
per-relation time travel, and bindings for C and other languages.
*Relation:* close to an Upshot storage backend (Rust, embeddable, C bindings, versioned relations); per-relation
opt-in history is a metadata-style choice for VER-1.
<https://github.com/cozodb/cozo>

**LogicBlox / LogiQL.** A commercial Datalog platform for whole enterprise applications; entities are created by
*refmode* and *constructor* predicates, a constructor mapping each combination of key arguments to one entity.
*Relation:* constructor predicates are a working form of value invention for derived entities (see §12, §13).
<https://www.relational.ai/post/design-and-implementation-of-the-logicblox-system>,
<https://github.com/KnowSciEng/doop/blob/master/docs/datalog-101.md>

**Rel (RelationalAI, SIGMOD 2025).** A relational language meant to describe an entire application's semantics, not
just queries, to remove the impedance mismatch with a host language.
*Relation:* shares Upshot's ambition of a data-first whole-application language; purely relational, while Upshot
keeps object structures.
<https://arxiv.org/abs/2504.10323>

**Morel (Julian Hyde).** Standard ML extended with relational expressions, compiled to relational algebra and
optimised by Apache Calcite.
*Relation:* a functional language whose collection expressions are optimised like queries, the role Upshot's
computed collections could play.
<https://github.com/hydromatic/morel>, <https://thestrangeloop.com/2021/morel-a-functional-query-language.html>

## §6 Reactive data for applications and sync engines

**Convex.** A reactive backend database where queries are deterministic TypeScript functions; it tracks each query's
read set and re-runs it and pushes results to subscribed clients when that data changes; mutations are serialisable
transactions, and side effects live in separate *actions*.
*Relation:* Upshot's split of computed values (deterministic, reactive), actions (transactions on sources) and exposed
APIs mirrors Convex's queries / mutations / actions. Determinism is what makes caching and recomputation safe.
<https://docs.convex.dev/understanding/>

**Electric Clojure (Hyperfiddle).** A reactive Clojure dialect in which one program spans client and server
(`e/client`, `e/server`); the compiler splits it into two reactive programs and streams values across the network.
Reactive loops over *differential collections* bind each child's state to an entity by a user-supplied key (like
React keys).
*Relation:* placement inside one program with automatic network plumbing; Upshot instead keeps placement out of the
program (RUN-1). Keyed differential collections are another identity precedent (§13).
<https://github.com/hyperfiddle/electric>

**Riffle (Schiefer, Litt, Schickling, Jackson, 2022).** Puts all app state, including UI state, in a local reactive
relational database (SQLite + React), so the whole app is a graph of incrementally maintained queries.
*Relation:* the same bet as Upshot (declarative data model, reactive computed data) applied to local-first UI;
storing UI state alongside domain data is one way to give source values to derived entities.
<https://riffle.systems/essays/prelude/>

**Linear's sync engine.** Linear's client holds an object graph of models in an *object pool* keyed by UUID, observed
through MobX; edits are transactions sent to the server and confirmed by delta packets; some collections load lazily.
*Relation:* industrial evidence that an object graph with references, lazy collections and transactional actions
gives feature velocity; identity is always a stored UUID.
<https://linear.app/now/scaling-the-linear-sync-engine>, <https://github.com/wzhudev/reverse-linear-sync-engine>

**Zero (Rocicorp).** A query-driven sync engine: the client writes ordinary queries (ZQL), the server keeps their
results live with incremental view maintenance over a replica of Postgres, and the client caches and answers locally.
*Relation:* "what to sync and cache is decided by the queries in use" is a usage-driven physical decision, like
OPT-4's metadata from usage metrics.
<https://zero.rocicorp.dev/docs/introduction>

**Instant.** A triple-store backend with a graph query language (InstaQL) compiled to Datalog, with optimistic local
updates and reactivity.
*Relation:* the argument in its founding essay, that front-end data problems are database problems, is Upshot's
LANG-5 from the web side.
<https://www.instantdb.com/essays/next_firebase>

**Skip (SkipLabs).** A reactive framework (TypeScript API, native runtime written in the Skip language) in which
services are built from *collections* (keys to multiple JSON values) and *mappers* (deterministic functions from
input entries to output key/value pairs), plus reducers. Collections are **eager** (always kept up to date) or
**lazy** (computed per key on demand); *resources* expose parameterised outputs to clients; SKDB is a reactive SQL
database on the same engine.
*Relation:* one of the closest systems: computed collections defined by mappers, eager vs lazy as a per-collection
choice, services with typed outputs akin to zones with APIs. Identity of an entry is its key, computed by the
mapper (§13). In Upshot the eager/lazy choice would be metadata rather than part of the program.
<https://github.com/SkipLabs/skip>

**MobX and Signals.** MobX computed values are derived from observables and cached while observed; when no reaction
observes them they are *suspended* and recomputed on each read, unless `keepAlive` is set. The TC39 Signals proposal
standardises `Signal.State` and `Signal.Computed` with the same lazy, pull-based semantics.
*Relation:* the simplest model of Upshot's computed values in memory; "suspend when unobserved" vs `keepAlive` is a
store-vs-recompute decision already exposed as a knob.
<https://mobx.js.org/computeds.html>, <https://github.com/tc39/proposal-signals>

## §7 Keyed reconciliation in UI frameworks

**React.** State belongs to a component's *position* in the tree; it is preserved while the same component renders at
the same position and destroyed when it is removed. A `key` becomes part of the position, so items keep their state
when reordered, and changing the key resets the state. To keep state across removal, the docs list hiding instead of
removing, lifting the state to the parent keyed by item, or keeping it in an external store.
*Relation:* React already offers the three lifetimes Upshot needs: state that dies on removal (default), state that
survives removal (lifted into a store keyed by identity), and fresh state on each appearance (change the key).
<https://react.dev/learn/preserving-and-resetting-state>

**Elm `Html.Keyed` and Svelte keyed `each`.** Both pair list items with unique keys so the framework moves, inserts
and deletes the right nodes instead of diffing by position; Svelte notes that string or number keys let identity
persist when the objects themselves change.
*Relation:* same lesson: positional identity is the default and is wrong for reorderable collections; explicit keys
derived from the data fix it.
<https://guide.elm-lang.org/optimization/keyed>, <https://svelte.dev/docs/svelte/each>

## §8 Automatic physical design: self-tuning, learned and LLM-driven

**Self-driving database systems (Pavlo et al., CIDR 2017; Peloton, NoisePage).** A DBMS that forecasts its workload
and applies physical-design actions (e.g. add an index) on its own, rather than advising a human.
*Relation:* the long-term form of OPT-4's "generated from usage metrics"; its forecasting and action-cost models are
a reference for how Upshot could collect metrics and regenerate metadata.
<https://web.eecs.umich.edu/~linmacse/publications/2017.self-driving.cidr.pdf>

**Automatic indexing in Azure SQL Database (SIGMOD 2019).** A production service that recommends, creates,
validates and reverts indexes for millions of databases, checking that each change actually improved performance.
*Relation:* the validation loop (apply, measure, revert) is the safety net Upshot needs if metadata is generated
automatically; it works because indexes cannot change results, the same guarantee as OPT-2.
<https://www.microsoft.com/en-us/research/publication/automatically-indexing-millions-of-databases-in-microsoft-azure-sql-database/>

**Learned index structures (Kraska et al., SIGMOD 2018).** Treats indexes as models from key to position and replaces
B-trees, hashes and Bloom filters with learned models where the data allows.
*Relation:* evidence that the physical layer can be specialised to the data it holds, behind an unchanged logical
interface.
<https://research.google/pubs/the-case-for-learned-index-structures/>

**LLM-driven database tuning.** GPTuner (SIGMOD 2024) uses an LLM to read manuals and guide Bayesian optimisation of
configuration knobs; an experimental evaluation (Li et al., 2024) finds LLMs competitive at knob pruning and
recommendation without training.
*Relation:* OPT-4's LLM-generated metadata has early evidence for databases. Upshot's advantage is that its metadata
can be checked: it cannot change results, so an LLM proposal is safe to try and keep only when measured faster.
<https://arxiv.org/abs/2311.03157>, <https://arxiv.org/abs/2408.02213>

## §9 Versioned data

**Datomic.** A database of immutable facts (datoms) with transaction time; a database value can be queried *as of*
any past point; writes produce new database values through a single transactor.
*Relation:* "database as a value" is the natural reading of VER-1; a version per action (transaction) is the model to
start from.
<https://docs.datomic.com/>

**XTDB.** A SQL database where every table is *bitemporal*: system time (when recorded, immutable) and valid time
(when true, editable), queried with SQL:2011 `FOR SYSTEM_TIME AS OF` / `FOR VALID_TIME AS OF`.
*Relation:* if Upshot ever needs corrections of the past distinct from history of recording, bitemporality is the
known design; atemporal by default keeps it out of the way.
<https://docs.xtdb.com/about/time-in-xtdb.html>

**Dolt and TerminusDB.** Git-style version control of data: Dolt stores SQL tables in content-addressed *prolly trees*
with branches, merges and cheap diffs; TerminusDB stores graphs as immutable delta layers per commit.
*Relation:* structural sharing between versions makes "all data is versioned" affordable; branches and merges are a
possible extension of VER-1.
<https://docs.dolthub.com/architecture/storage-engine>, <https://github.com/terminusdb/terminusdb>

**Timely dataflow progress tracking.** Each operator knows a *frontier*, the lower bound of timestamps that may still
arrive, so a reader can ask whether the result for a version is complete.
*Relation:* the mechanism behind VER-2 ("the latest is not available yet", "being recomputed") in an incremental
engine.
<https://dev.materialize.com/api/rust/timely/progress/index.html>

## §10 Multi-tier and tierless languages; actors with placement as configuration

**Hilda (Cornell, ICDE 2006).** A declarative language for data-driven web applications with one relational data
model across all tiers, built from *AUnits* (structured, encapsulated application units); a follow-up partitions
logic automatically between client and server.
*Relation:* data-centric whole-application language whose units resemble zones; automatic partitioning is a
generated placement.
<https://www.cs.cornell.edu/people/jai/papers/Hilda.pdf>

**Links (Cooper, Lindley, Wadler, Yallop, 2006).** One program compiled to JavaScript for the client, server code,
and SQL for the database ("web programming without tiers").
*Relation:* the founding tierless language; Upshot differs by keeping placement out of the program.
<https://www.cl.cam.ac.uk/~jdy22/papers/links.pdf>, <https://links-lang.org/>

**Eliom / Ocsigen.** OCaml extended with client/server annotations, static slicing and type-safe communication,
with tierless modules.
*Relation:* modularity of tierless code (separate compilation of tier-annotated modules) is relevant to compiling
zones separately.
<https://arxiv.org/abs/1901.11411>

**Multitier programming survey and ScalaLoci.** A survey of the design space (placement by annotations, types or
analysis); ScalaLoci expresses placement with *placement types* and declares architecture (peers and ties) at the
type level.
*Relation:* a map of the alternatives to Upshot's choice of placement-as-configuration.
<https://decomposition.al/CSE290S-2023-01/readings/multitier-survey.pdf>

**Unison.** A content-addressed language (definitions identified by hash) whose distributed model moves computations
and their dependencies between nodes as ordinary values.
*Relation:* content addressing makes code location-independent; relevant if Upshot zones move between runtimes.
<https://www.unison-lang.org/docs/the-big-idea/>

**Orleans virtual actors (MSR 2014).** Actors (*grains*) that always exist logically and are activated on demand on
some server; *placement strategies* (random, prefer-local, hash-based, activation-count, custom) are set per grain
class or globally, outside the grain's logic.
*Relation:* closest runtime precedent for "each zone is assigned to a runtime by configuration" (RUN-1); virtual
identity (an actor exists whether or not it is active) is also a lifetime model.
<https://www.microsoft.com/en-us/research/publication/orleans-distributed-virtual-actors-for-programmability-and-scalability/>,
<https://learn.microsoft.com/dotnet/orleans/grains/grain-placement>

**Erlang/OTP.** Processes communicate only by messages, and message passing to processes on other nodes is
transparent when pids are used.
*Relation:* the classic proof that "interact only through each other's interface" (ZONE-2) makes placement a
deployment detail.
<https://www.erlang.org/doc/system/distributed.html>

## §11 Bloom, CALM and Hydro (Berkeley)

**Bloom and the CALM theorem (CIDR 2011).** Bloom is a Datalog-derived language for distributed programs over
unordered collections; CALM states that logically monotonic programs are eventually consistent without
coordination, and Bloom's analysis finds the "points of order" that need it.
*Relation:* if Upshot zones run on different runtimes, CALM tells which cross-zone computed values can stay consistent
without coordination (monotonic ones such as counts of growing sets) and which cannot.
<https://www.cidrdb.org/cidr2011/Papers/CIDR11_Paper35.pdf>

**"New Directions in Cloud Programming" (CIDR 2021).** Proposes separating a distributed program into a *PACT* of four
facets: Program semantics, Availability, Consistency and Targets for optimisation, compiled by a multi-stage compiler
that emits readable, hand-tunable code at each stage.
*Relation:* the closest statement of Upshot's architecture for distributed systems: logic declared once, with
placement and performance targets as separate facets. Upshot's optimisation language corresponds to the A, C and T
facets.
<https://arxiv.org/abs/2101.01159>

**Hydroflow and Hydro.** Hydroflow (OOPSLA 2023, now DFIR) is a Rust dataflow runtime with a lattice-based IR;
Hydro is a Rust framework where one program spans processes and clusters, with locations as types
(`Process`, `Cluster`) and compile-time checks of ordering. Related work applies *decoupling* and *partitioning*
rewrites to distributed protocols safely (SIGMOD 2024).
*Relation:* Rust, a dataflow core, and correctness-preserving rewrites that move logic between machines are what an
Upshot multi-runtime future needs. Hydro puts locations in the program's types; Upshot keeps them in metadata, so
Hydro's rewrites are the kind of thing an Upshot optimiser could apply.
<https://hydro.run/>, <https://github.com/hydro-project/hydro>, <https://arxiv.org/abs/2404.01593>

## §12 Object creation in declarative languages

**IQL (Abiteboul & Kanellakis, SIGMOD 1989).** Object identifiers as a query-language primitive: a rule-based
language over a model allowing cycles in schemas and instances, which *invents* new oids in rule heads.
*Relation:* the foundational treatment of computed data that creates objects with identity (and cyclic structures,
MODEL-1). Invented oids are fresh per evaluation, the "fresh on each recompute" lifetime.
<https://cs.brown.edu/research/pubs/techreports/reports/CS-89-26.html>

**ILOG (Hull & Yoshikawa, VLDB 1990).** Declarative creation of object identifiers by **Skolem functors**: a created
object's identity is a function of the values that caused its creation, so re-deriving from the same values yields
the same object.
*Relation:* the theory of deterministic identity for objects created by a computed collection: identity = functor
applied to the creating key (e.g. `container(parent, i)`), stable across recomputation (§13).
<https://www.vldb.org/conf/1990/P455.PDF>

**Surrogate keys and slowly changing dimensions (data warehousing).** A surrogate key is a generated identity kept in
a mapping from natural key to surrogate; in a type 2 dimension each version of an entity gets its own surrogate.
*Relation:* the industrial version of "survives leaving and re-entering": the natural-key → identity map persists
independently of whether the entity is currently present, and versions are explicit (VER-1).
<https://en.wikipedia.org/wiki/Slowly_changing_dimension>

## §13 Identity and lifetime in computed collections

The problem (plan, open questions): a computed collection creates objects (e.g. containers computed from a count),
those objects carry *source* values (e.g. each container's `oranges`), and when the collection is recomputed Upshot
must know which new object is "the same" as an old one, and what happens to its source values when it leaves the
collection and possibly comes back. Three lifetimes must be supported: **survive** (values persist across leaving and
re-entering), **die with it** (values are dropped when the object leaves), **fresh** (every recompute yields new
objects).

What prior work offers:

1. **Identity is a key, not an address.** Every working system identifies created objects by a key computed from
   data: Skip mappers emit keys; React, Elm, Svelte and Electric use user-supplied keys; Boomerang's resourceful
   lenses align chunks by keys; LogicBlox constructors and ILOG Skolem functors make identity a function of the
   creating arguments; Salsa's interned structs are identified by their field values. Upshot's computed
   collections should therefore *declare* the identity of their elements (a key expression), with the creating
   context (owning object, collection) implicitly part of it, as in Salsa (creating query) and React (parent
   position).
2. **Positional identity is the fallback, and it is fragile.** React's default (position among siblings), Salsa's
   disambiguator (n-th struct with this hash in this query execution) and counter-based names in Adapton all work but
   misidentify objects when elements are inserted or reordered. For "containers from a count", the index *is* the
   natural key, so positional identity is correct there; for filtered or sorted collections it is not.
3. **Lifetime is a separate choice from identity.** With deterministic identity, the three lifetimes differ only in
   what happens to source values keyed by that identity:
   - *die with it*: drop them when the key leaves (Salsa tracked structs are deleted when not re-created; React
     destroys state on removal; Svelte keyed blocks destroy the component);
   - *survive*: keep them in a store keyed by identity, independent of membership (React's "lift state up" or
     external store, warehouse surrogate-key maps, Riffle keeping UI state in the database, Orleans' virtual actors
     that exist logically whether or not active, MobX `keepAlive`). This needs a retention policy: forever, until an
     action deletes them, or bounded by versioning (VER-1);
   - *fresh*: make the recompute itself part of the identity (React: change the key; IQL: invent new oids). For
     incremental engines this must still be deterministic, for example identity = (key, version of the recompute), or
     results would differ between an incremental and a from-scratch run.
4. **Determinism is required by the "metadata never changes results" rule.** Adapton's *from-scratch consistency* and
   DBSP's and differential dataflow's algebra both assume that a recomputation produces the same keys as a full
   evaluation. Fresh, non-deterministic identities (IQL-style invention, auto-increment counters) would make results
   depend on whether a value was stored or recomputed, which OPT-2 forbids. So even "fresh" objects need an identity
   defined by the program, not by the evaluation order.
5. **Names for nested and dynamic creation.** Nominal Adapton shows that letting the programmer name allocations
   inside a computation gives precise reuse when structure changes; Salsa's identity fields play the same role.
   Upshot can make the key expression optional, defaulting to positional identity within the creating object, and
   require it where the collection is filtered or reordered.
6. **Where the source values live is metadata.** Whether surviving values are stored on the object, in a side table
   keyed by identity, or in the database is a physical choice; the lifetime (survive, die, fresh) is a semantic choice
   and belongs to the feature language. This keeps the division of LANG-6 and OPT-2.
7. **Gaps.** No surveyed system offers all three lifetimes as declared alternatives on a computed collection, and none
   treats retention of surviving values together with versioning. Incremental view maintenance of Datalog with value
   invention exists for bounded Skolem terms, but practical engines (DBSP, differential dataflow) leave object
   creation to keys computed by user code. This looks like genuinely new ground for Upshot.

## §14 Takeaways for Upshot

- Upshot's core idea has a strong pedigree: Codd's data independence (§1), Out of the Tar Pit's essential logic vs
  performance hints (§1), Halide's algorithm vs schedule (§2) and Hydro's PACT (§11). The novelty is applying it to
  a program's in-memory object structures (cycles, collection references) and runtime placement together, with
  metadata generated from metrics or by an LLM.
- Make "metadata never changes results" a checkable property: define from-scratch semantics, and require the
  incremental runtime to agree with it (Adapton's from-scratch consistency, DBSP's derivation of incremental forms).
  This also makes generated metadata safe to try and revert (§8).
- For the prototype runtime in Rust (RUN-2), study Salsa (identity rules, revisions, early cutoff) and DBSP or
  differential dataflow (computed collections as differences); Salsa or a DBSP-style core may be reusable.
- Treat eager vs lazy, stored vs recomputed, partial materialisation and indexes as metadata choices per computed
  value (Skip, Noria, MobX `keepAlive`, Dyna's learned strategies).
- Declare bidirectional values with lens combinators and the round-trip laws; for collections, use keyed (resourceful)
  or relational lenses, and fix the complement so a write back to sources is unambiguous (§4).
- Give computed collections a declared element identity (a key, defaulting to position within the creator) and a
  declared lifetime for their elements' source values (survive, die with it, fresh), with all identities
  deterministic (§13).
- Version per action, as database values (Datomic), with structural sharing (Dolt, TerminusDB); use frontier-style
  progress information for self-inspection (§9, VER-2).
- Keep zone placement as configuration (Orleans, Legion mappers); if zones ever span runtimes, use CALM to decide
  which cross-zone computed values need coordination (§10, §11).
