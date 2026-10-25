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
