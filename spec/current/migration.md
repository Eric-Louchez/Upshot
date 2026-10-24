# Migration — current specification

How existing data follows changes to the program. The design study is [`plan/migration.md`](../../plan/migration.md).

Derived from the log; where it disagrees with the log, the log wins (spec/README.md).

## Scope

**MIG-1** Upshot has an integrated migration story: when the feature-language program changes (its data structures,
sources, computed values or constraints), the existing, versioned data is carried over to the new program as part
of Upshot itself.
Source: L427008, L427445

**MIG-2** Migration covers every kind of program change, not only renames and moves (which persistent ids would
handle, L427115): among them changing actions, constraints and computation functions, converting between source and
computed values, adding source and computed values, and changing APIs.
Source: L427237

## Program versions and the migration transform

**MIG-3** A program can hold parallel versions of its code: a snapshot of the whole program is kept for each version
that migrations must support. Upgrades are hot, with no downtime: the migration is a single transaction between
actions (MIG-6), not a stop of the system (interpretation: J427505).
Source: L427445, L427505, J427505

**MIG-4** Each new version comes with a **migration transform** from the old program to the new one: it migrates the
old version's data zones to the source values of the new version's data zones. With metaprogramming (META-1) a
migration is a metaprogramming operation; without it, migration is a built-in operation of the same kind
(interpretation: J427445).
Source: L427445, J427445

**MIG-5** A source value of the new version is migrated automatically when its matching value in the old version has
the same format, data type and structure, even if the old value is a computed value; the transform only has to say
how the other source values are obtained (interpretation: J427447).
Source: L427445, J427447

**MIG-6** The migration is an atomic transaction that runs like a normal action, with its own transaction version,
checked against the new version's constraints (ACT-3, CONS-2; interpretation: J427448). Actions before the migration
apply to the old data schema; actions after it apply to the new one. Old and new versions never both accept actions
on the same data (interpretation: J427505).
Source: L427445, J427448, L427505, J427505

## History

**MIG-7** A migration never rewrites the version history. After it, some values may no longer exist (as identified by
their instance id) and new values may exist with no history from before the migration; neither is new to the system.
Source: L427445
