# Model — current specification

The data a feature-language program describes: its data structures, data sources and computed values. How the data
is held is the optimisation metadata's concern ([`optimisation.md`](optimisation.md)).

Derived from the log; where it disagrees with the log, the log wins (spec/README.md).

## Data structures

**MODEL-1** The feature language lets a program specify its data with intuitive data structures chosen for
maintainability and feature development, not efficiency. They can include circular references, collections and
collection references, and other forms that are typically not possible in a database.
Source: L426464

## Data sources and computed data

**MODEL-2** The feature language marks some data as **data sources**; all other data is **computed**. A data source
holds data as given; computed data is defined over other data and stays consistent with it (interpretation:
J426466). The program's design always specifies its sources, even where two-way computed values let the runtime
persist another form (OPT-3).
Source: L426464, J426466, L427008

**MODEL-3** Computed values include counts, averages and arbitrary computed values, and computed collections.
Source: L426464

**MODEL-4** Some computed values can be **bidirectional**: they have a two-way computed path to their data sources,
so either the value or its sources can be the persisted form, and writing to the value is carried back to its
sources (interpretation: J426467). Which form is persisted is decided by the optimisation metadata (OPT-3).
Source: L426464, J426467

**MODEL-8** A computed value is writable only if it is two-way all the way back to all of its sources; any other
computed value is read-only (interpretation: J427010).
Source: L427008, J427010

**MODEL-10** Within one transaction, a computed value cannot have a run-time circular dependency: an evaluation
that needs a value to compute that same value fails (ACT-4). Fixed-point dependencies, resolved by iterating until
nothing changes, are forbidden (interpretation: J428306).
Source: L428306, J428306

**MODEL-11** A circular dependency in the program's static dependency graph is an error unless it is explicitly
annotated, in which case it is allowed; MODEL-10 still holds for every evaluation (interpretation: J428307).
Source: L428306, J428307

**MODEL-9** Actions can convert a computed value into a source value, keeping its value and disconnecting it from
what it was computed from, and, in principle, a source value into a computed value (interpretation: J427009). Any
computed value can be converted to a source this way.
Source: L427008, J427009

## Objects of computed collections

**MODEL-5** A computed collection can create the objects it holds, and those objects can carry source values (e.g.
a collection of containers computed from a source count, each container with a source number of oranges).
Source: L426551

**MODEL-6** The feature language lets a program specify each object's identity within a computed collection; the
identity decides which source values an object has across recomputations (interpretation: J426552).
Source: L426551, J426552

**MODEL-7** Three semantics for the source values of such objects are equally supported: (A) they belong to the
object's identity and survive the object leaving and re-entering the collection; (B) they end when the object leaves
the collection; (C) objects keep no identity across recomputations, so each recompute yields fresh objects
(interpretation: J426551). Whether the lifetime of these objects is stated in the feature language or is an
optimisation concern only is not yet decided (`plan/README.md`).
Source: L426551, J426551
