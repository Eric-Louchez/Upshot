# Relaxation — current specification

Relaxation annotations: how a program trades the feature language's built-in promises for performance.

Derived from the log; where it disagrees with the log, the log wins (spec/README.md).

**REL-1** The feature language builds certain promises directly into the structure of a program (such as atomicity,
exact results and always-current computed values). Those promises can stop the optimiser from delivering some
performance profiles that hand-built code reaches.
Source: L428000

**REL-2** Any performance deviation between an Upshot program and hand-built code can be addressed with **relaxation
annotations** on the feature-language program, each relaxing one promise where it is written (interpretation:
J428000). This is how LANG-9 is met.
Source: L428000, J428000

**REL-3** Relaxations are never applied without the developer's knowledge: they may degrade functionality, so the
developer assesses each as a trade-off and states it in the program. The optimiser alone never relaxes a promise
(OPT-2, OPT-9).
Source: L428000

## Validating only the values of interest

**REL-4** Checking every constraint and every possible run-time cycle before each transaction commits can cost far
more than the transaction, especially when each transaction changes much data but only sparse reads matter. An
action can therefore carry an annotation naming a subset of data values of interest; only the path along those
values is validated (interpretation: J428329).
Source: L428329, J428329

**REL-5** Under that annotation, values outside the validated path can end up failed (a broken constraint or a
run-time cycle). A read that meets a failed value fails, and an action that reads a failed value fails. The
transaction that caused it is not rolled back, unless its annotation says it is (interpretation: J428330).
Source: L428329, J428330

**REL-6** Failed values are repaired either by a rollback, possibly of several transactions until the real culprit
is found, or by a transaction that fixes the root cause in the data without reading any of the failed values. A
rollback undoes a transaction as if it never happened, and the actions after it are replayed (interpretation:
J428350).
Source: L428329, L428350, J428350

**REL-7** A program can read the state of a failed value (that it is failed, and why) without failing, so a data
zone can, if so desired, be made significantly resilient to errors (interpretation: J428351).
Source: L428350, J428351
