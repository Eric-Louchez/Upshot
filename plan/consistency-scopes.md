# Consistency scopes — partitioning atomicity inside a zone

The problem (L434366). Hand-built systems scale by splitting an application into **entities** (a user, an order,
an account; Helland's *Life beyond Distributed Transactions*), each atomic on its own, talking by messages, with no
atomicity across them. In Upshot those entities are **items inside one zone**, and a zone is atomic as a whole
(ACT-3, MIG-6). Splitting it looks like an optimisation, but:
- it changes results where an action, a constraint or a computed value spans entities (a transfer between two
  accounts, a username unique across users, a total over all orders), so OPT-2 keeps it out of the metadata;
- it depends on the details of the data structures (which item owns which, what an action reaches), which the
  optimiser is meant to own physically and the feature language logically.

Everything below is a proposal.

## 1 Split the problem in two

Partitioning has a part that changes nothing and a part that changes results. They can be separated, and only the
second needs the developer.

**What changes nothing: partitioning by footprint.** An action's *footprint* is what it reads and writes. When two
actions' footprints are disjoint, running them on different partitions, in any order, gives the same results as
the single serial sequence. So the optimiser can partition a zone wherever footprints allow, with no annotation:
- *Static footprints*: the compiler derives, for each action, the data it can reach from its inputs (an action
  `addItem(order, …)` reaches that order's subtree and nothing else). Entities then **emerge from the program**: an
  entity is a region that actions' footprints never cross. The developer never names them.
- *Dynamic footprints*: where static analysis cannot bound a footprint, the runtime finds it by running the action
  (deterministic databases such as Calvin do this with a "reconnaissance" pass) or optimistically, retrying on
  conflict.
- *Actions that do cross regions* still run atomically: the runtime coordinates them (a deterministic global order
  for those actions only, or a distributed commit). That costs latency for those actions, never correctness.
  Deterministic databases show that this keeps throughput high when crossing actions are rare.
- *Constraints and computed values* that span regions (a global count, uniqueness) are footprints too: they read
  across regions, so they force the coordination they need, and only that.

This gives the hand-built entity structure's throughput wherever the program's own actions respect entity
boundaries, which in practice is most of the time (L434366's observation: transactions mostly stay on their own
user's track).

**What changes results: weakening atomicity across regions.** What hand-built systems also gain, and footprint
partitioning cannot, is never coordinating across entities at all: a transfer done as two local steps with a
compensation (a saga), a cross-entity total that lags, a uniqueness check that is reserved and confirmed later.
These change what a program can observe, so they are relaxations (REL-1 to REL-3), written by the developer.

## 2 Consistency scopes: the relaxation, written in feature terms

The relaxation must not mention partitions, placement or layouts (those belong to the optimiser and would break
the abstraction). It can be written entirely in the feature language's own vocabulary, items and collections:

- **A consistency scope** is declared on a collection: `scope per item of accounts` says each account, with what it
  owns (its own fields, its owned sub-objects), is a unit of atomicity. Nothing is said about where it lives. With
  no scope declared, the whole zone is one scope (today's strict semantics).
- An **action whose footprint stays inside one scope** behaves exactly as now.
- An action that **crosses scopes** must say how, as part of its definition:
  - *atomic* (the default): the runtime coordinates, as in §1, at a latency cost;
  - *saga*: it is split into one step per scope it touches, each atomic, with a declared compensation for each step
    (ACT-7); between steps other actions can see the intermediate state;
  - *forbidden*: crossing is a compile-time or run-time error, so a scope boundary is a real guarantee.
- A **computed value or constraint that crosses scopes** says whether it is *exact* (coordinated, the default),
  *lagging* (computed in the background with a propagation status, VER-2), or for constraints *checked later* with a
  declared repair (the values-of-interest relaxation, REL-4 to REL-6, is one form of this).

The optimiser then has what it needs: scope boundaries are places where it may split without coordination, because
the program has said what crossing them means. It still chooses the physical partitioning (which scopes go
together, on which runtime), and may keep several scopes together when that is cheaper.

Why this fits the abstraction:
- The **feature language** states meaning: which groups of items must change together, and what an action that
  touches several groups means. That is a domain decision (it is the "aggregate" of domain-driven design, a
  consistency boundary chosen for business reasons), so the developer is the right person to make it (REL-3).
- The **optimiser** owns every physical decision: partitions, placement, coordination protocol, and the footprint
  analysis that lets it partition further than the declared scopes when no action crosses.
- The two meet only through feature-level names (collections, items, actions), not through data-structure details.

## 3 Helping the developer choose

Scopes are a trade-off the developer must accept, but the runtime can show where they would pay:
- runtime analytics record which actions cross which regions and how often (`transfer` crosses two accounts in
  0.1% of calls; `rename` crosses users through the uniqueness constraint in every call);
- the optimiser (or an LLM over those analytics) proposes a scope and says what it would buy and what it would
  change ("declare `scope per item of accounts` and make `transfer` a saga: throughput ×20, transfers visible
  half-done for a few milliseconds");
- the developer accepts the proposal into the feature program, or not.

## 4 Questions for the user

1. Does the split hold: footprint partitioning as a pure optimisation, and scopes as the only feature-level
   relaxation of atomicity inside a zone?
2. Should scopes be declared on collections (`per item of accounts`), on types (`Account is a scope`), or both?
3. Default for an action that crosses scopes: *atomic* (coordinated, slower) or *forbidden* until the developer
   chooses?
4. How "what an item owns" is defined: by the structure (owned fields and sub-objects, as opposed to references),
   or declared?
