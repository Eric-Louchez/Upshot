# Zones — current specification

Data zones: the separate areas a program's data is divided into, and how they interact. Where each zone runs is in
[`runtime.md`](runtime.md).

Derived from the log; where it disagrees with the log, the log wins (spec/README.md).

**ZONE-1** The feature language lets a program define **data zones**: separate areas of its data.
Source: L426531

**ZONE-2** Different zones interact only through each other's APIs (API-1). Every piece of data belongs to exactly
one zone and each API to a zone; one zone's data holds no direct reference into another zone's data
(interpretation: J426532).
Source: L426531, J426532
