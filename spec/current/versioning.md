# Versioning — current specification

Versions of the data and the dataset's view of its own state.

Derived from the log; where it disagrees with the log, the log wins (spec/README.md).

**VER-1** All data is versioned: every change to data produces a new version, and a reader can tell which version
of a value it sees (interpretation: J426469). Whether and how long earlier versions stay readable is not yet
specified.
Source: L426464, J426469

**VER-2** Ideally the dataset can inspect itself: a program can query the state of its own data, such as data being
recomputed or the latest version not being available yet (interpretation: J426470). This is a goal, not yet a hard
requirement.
Source: L426464, J426470

**VER-3** Values have a version history, and a program has access to it. Whether the history is read directly or as
an explicit list through observables is not yet decided (`plan/README.md`).
Source: L427008
