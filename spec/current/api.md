# API — current specification

The APIs a feature-language program specifies: what they expose and how. Data zones, which interact only through
APIs, are in [`zones.md`](zones.md).

Derived from the log; where it disagrees with the log, the log wins (spec/README.md).

**API-1** The feature language lets a program specify one or more APIs.
Source: L426531

**API-2** An API is composed of **exposed values**, computed or source (MODEL-2), and **exposed actions** (ACT-1).
Source: L426531

**API-3** An exposed value can be exposed as read/write; otherwise it is read-only. A read/write exposed value is a
data source or a bidirectional computed value (MODEL-4), since a write follows the action rules (ACT-2); a one-way
computed value can only be exposed read-only (interpretation: J426531).
Source: L426531, J426531
