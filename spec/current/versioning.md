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

**VER-3** Values have a version history, and the feature language gives full access to the historical versions of
each value: those of the single canonical history (VER-6). Whether the history is read directly or as an explicit
list through observables is not yet decided (`plan/README.md`).
Source: L427008, L429758

**VER-4** Migrations cannot rewrite history (MIG-7).
Source: L427445, L429758

**VER-5** History can be rewritten by undoing or reshuffling applied transactions: the data is rolled back to before
the transaction to undo and all the transactions after it are replayed (REL-6). App admins need to be able to do this
when they want to (interpretation: J429758), through functionality the runtime offers: the admin controls an app's
behaviour through the runtime.
Source: L428350, L429753, L429758, J429758, L429771

**VER-7** A history rewrite must support the scenario where every value already sent to a third party stays unchanged,
even if the id of the transaction it came from changes (interpretation: J429771). Deciding whether a rewrite preserves
them cannot be left to the runtime's judgement alone; how it is decided is open (`plan/README.md`).
Source: L429771, J429771

**VER-6** After a rewrite there is still a single canonical history, the one the app sees. Whether the runtime holds
on to the alternate histories is up to the runtime; from inside the app there is no way to access them.
Source: L429758
