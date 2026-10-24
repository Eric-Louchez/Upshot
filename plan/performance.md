# Performance — what Upshot's hard constraints rule out

LANG-9 (L427529): Upshot should, in theory, reach the best hand-tuned performance, so every optimisation that its
hard constraints rule out must be addressed. This study lists those optimisations and proposes how each could be
recovered. Everything here is a proposal unless a rule is cited.

## 1 What the optimiser can already do alone

Anything whose outcome is identical to the strict semantics: concurrent execution of independent actions,
speculation with rollback, value prediction, hot-path specialisation (OPT-8); stored vs recomputed values, eager vs
lazy, indexes, layouts, caching that is always invalidated, memoisation of configuration-value instances (OPT-2,
OPT-7); inlining across zones on the same runtime; incremental constraint checking; copy-on-write versions;
retention of history nobody can read (plan, identity and lifetime).

Two limits even here:
- **Speculation cannot cross an external effect.** A guessed value that reaches a client, the harness or another
  runtime cannot be rolled back. Outputs must be held until the speculation is confirmed (the output-commit problem),
  which caps how far speculation decouples successive transactions.
- **Errors and non-termination are results.** If a computed value fails or never finishes, eager and lazy evaluation
  differ (eager fails, lazy may never look). For the optimiser to choose freely, a failing computation has to be a
  value the semantics defines (ACT-4), not a crash of whoever evaluated it.

## 2 Hard constraints and what they rule out

| Hard constraint | Optimisations it rules out (in hand-tuned systems) |
|---|---|
| **One serial sequence of atomic actions** (ACT-3, MIG-6) | Stale reads (caches with a TTL, read replicas, follower reads); snapshot isolation and read committed (accepting write skew); per-entity or per-shard ordering instead of one global order; asynchronous replication; sagas (compensation) instead of distributed atomic commits; offline and local-first writes merged later |
| **Constraints always hold, checked per action** (CONS-1, CONS-2) | Skipping invariant checks on hot paths ("the caller guarantees it"); deferred or batched checking with later repair; sampling checks; uniqueness enforced only per shard |
| **Computed values always consistent with their sources** (MODEL-2) | Asynchronously refreshed views and search indexes; counters and denormalised copies updated in the background; periodic recomputation of expensive aggregates |
| **Metadata never changes results** (OPT-2), so results are exact and deterministic | Approximate aggregates and sketches (HyperLogLog, sampling); approximate nearest-neighbour indexes for vector search; reduced precision and quantisation; lossy compression; floating-point reassociation (parallel or SIMD/GPU reductions give slightly different sums); load shedding and degraded answers under load; "any element will do" choices made by whichever thread wins |
| **Deterministic identity** (MODEL-6, MODEL-7) | Identities from counters or clocks allocated per thread or node without coordination |
| **All data versioned, history never rewritten** (VER-1, VER-4) | Overwriting in place with no history for hot values; merging several actions into one version (group commit that hides intermediate versions); coalescing or throttling what observers see |
| **Zones meet only through APIs** (ZONE-2) | Sharing memory or a database across modules; an action that atomically changes two zones (if APIs do not offer that) |
| **Zones on different runtimes stay consistent** | Choosing availability over consistency during a network partition (AP systems): with strict semantics a partitioned zone must stop serving |

## 3 Proposal: relaxations are declared result sets

Each optimisation of §2 changes results, so OPT-2 forbids the optimiser from applying it alone (OPT-9). The proposal
keeps OPT-2 intact by changing what "the result" is: the feature language can declare, per value, action or API, a
**set of acceptable results** instead of a single one. The optimiser then picks any member of the set: the strict
member when it is cheap, a relaxed one when it pays. Hand-tuned code makes the same trade, but implicitly and
everywhere; Upshot makes it explicit and local, and the default stays strict.

Candidate declarations, one per row of §2:

| Declaration (feature language) | Allows the optimiser to |
|---|---|
| **Bounded staleness** on a read: "this action may read `x` up to N versions or T seconds old" | Use caches, replicas and follower reads; run the action without waiting for earlier writes to `x` |
| **Weaker isolation** on an action: snapshot reads, or ordering only per key or per zone | Run more actions concurrently; shard the sequence of transactions |
| **Eventual computed values**: "`v` may lag its sources; read it with its propagation status" (VER-2, L427057) | Refresh views, indexes and aggregates in the background |
| **Tolerance** on a computed value: "within ε", "within 1% with 99% confidence" | Sketches, sampling, reduced precision, reassociated floating-point reductions |
| **Approximate retrieval**: "top-k with recall ≥ r" | Approximate nearest-neighbour and other approximate indexes |
| **Nondeterministic choice**: "any element of…", "in any order" | Let whichever thread or shard finishes first decide |
| **Deferred constraint**: "must hold by the end of the batch" or "eventually, repaired by action R" | Batch or defer checks; sagas with compensation instead of distributed atomic commits |
| **Assumed constraint**: "trusted, checked only in tests" | Skip the check in production (an explicit, visible risk) |
| **Coarse history**: "versions of `x` may be coalesced", or "no history for `x`" | Group commit, in-place updates, throttled observers |
| **Commutative actions**: "these actions commute" (or proven so) | Merge concurrent and offline writes without coordination (CRDT-style; CALM, research 01 §11) |
| **Availability preference** for a cross-runtime API: "keep serving stale data during a partition" | Choose availability over consistency per API |
| **Degraded result** for an action under load: "may return the cached answer or reject" | Load shedding |

Commutativity is partly free: where the optimiser can prove that actions commute (counters, set inserts), it can
already merge them without changing results; the declaration only matters where it cannot prove it.

What the declared sets must preserve:
- **Every member is a result the strict program could explain**: a staler read is a read at an earlier version; an
  approximate sum has a declared bound. Testing can then run the strict member and the chosen member side by side.
- **Visibility**: a relaxed value carries its status (version read, staleness, error bound), so self-inspection
  (VER-2) shows it and programs can decide what to do.
- **Locality**: a relaxation applies where it is declared and does not leak; a value computed from a stale read is
  itself marked as stale.

## 4 Questions for the user

1. Is the "declared set of acceptable results" principle right, keeping OPT-2 as the rule that the metadata only
   chooses within what the program allows?
2. Which relaxations matter first? Bounded staleness, eventual computed values and tolerance look most common.
3. Should "assumed" (unchecked) constraints exist at all, given CONS-1's "must always hold"?
4. Floating point: is bit-exact reproducibility across runtimes (CPU, SIMD, GPU) required by default, or is a
   default tolerance acceptable?
5. Can an action change two zones atomically, or are cross-zone changes always separate actions (sagas)?
