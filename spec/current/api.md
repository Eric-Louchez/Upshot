# API — current specification

The APIs a feature-language program specifies: what they expose and how. Data zones, which interact only through
APIs, are in [`zones.md`](zones.md).

Derived from the log; where it disagrees with the log, the log wins (spec/README.md).

**API-1** The feature language lets a program specify one or more APIs; a data zone can expose several at once, e.g.
an old and a new version of an API (interpretation: J427517).
Source: L426531, L427516, J427517

**API-2** An API is composed of **exposed values**, computed or source (MODEL-2), and **exposed actions** (ACT-1).
Source: L426531

**API-3** An exposed value can be exposed as read/write; otherwise it is read-only. A read/write exposed value is a
data source or a bidirectional computed value (MODEL-4), since a write follows the action rules (ACT-2); a one-way
computed value can only be exposed read-only (MODEL-8; interpretation: J426531).
Source: L426531, J426531, L427008

**API-4** An API can declare **configuration values**: mandatory source values that the runtime or the harness
embedding it must provide whenever the API is used; in a web app, one of them is the current user. They belong to
that use of the API (a session or request), not to the zone's stored data (interpretation: J427077, confirmed by
L427098). They are not persisted, and they do not change during a use: a different value is a different use
(interpretation: J427099).
Source: L427076, J427077, L427098, J427099

**API-5** An API's exposed values and actions can be defined relative to its mandatory source values, so the API
presents a slice of its data zone that pivots on them (e.g. the current user's data).
Source: L427076

**API-6** Each distinct value of an API's configuration values yields its own parallel set of the values computed
from them. Those computed values are not needed while no use of the API with that configuration value is active;
whether they are cached, kept or recomputed every time is an optimisation concern (OPT-2).
Source: L427098
