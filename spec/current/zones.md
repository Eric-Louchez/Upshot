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

**ZONE-3** Suggested, not mandatory, separation for a web backend: two zones, a **user zone** and a **shared zone**,
both with the current user as a configuration value (API-4).
- The shared zone has access to all the backend's data; its API exposes only data the user is allowed to see, and
  its actions effectively execute with elevated privilege, so the actions its API exposes are only those safe for
  that user to take unrestricted.
- The user zone has direct access only to data the user is allowed to see, and can perform only actions the user is
  allowed to take; its API presents to the client all the data the client needs, in the shape it needs.
- The client uses the user zone's API only, and the user zone reaches shared data through the shared zone's API
  (interpretation: J427098).
Source: L427098, J427098
