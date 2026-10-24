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
J426466).
Source: L426464, J426466

**MODEL-3** Computed values include counts, averages and arbitrary computed values, and computed collections.
Source: L426464

**MODEL-4** Some computed values can be **bidirectional**: they have a two-way computed path to their data sources,
so either the value or its sources can be the persisted form, and writing to the value is carried back to its
sources (interpretation: J426467). Which form is persisted is decided by the optimisation metadata (OPT-3).
Source: L426464, J426467
