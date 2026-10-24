# Migration — current specification

How existing data follows changes to the program.

Derived from the log; where it disagrees with the log, the log wins (spec/README.md).

**MIG-1** Upshot has an integrated migration story: when the feature-language program changes (its data structures,
sources, computed values or constraints), the existing, versioned data is carried over to the new program as part
of Upshot itself. How is not yet decided (`plan/migration.md`).
Source: L427008

**MIG-2** Migration covers every kind of program change, not only renames and moves (which persistent ids would
handle, L427115): among them changing actions, constraints and computation functions, converting between source and
computed values, adding source and computed values, and changing APIs.
Source: L427237
