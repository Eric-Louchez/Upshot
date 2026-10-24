# Language — current specification

What Upshot is and why, its two parts, what it is implemented in, what it targets and what is out of scope. The
data a program describes is in [`model.md`](model.md); APIs in [`api.md`](api.md); data zones in
[`zones.md`](zones.md); runtimes in [`runtime.md`](runtime.md); the optimisation metadata in
[`optimisation.md`](optimisation.md); actions in [`actions.md`](actions.md); versions in
[`versioning.md`](versioning.md).

Derived from the log; where it disagrees with the log, the log wins (spec/README.md).

## Purpose

**LANG-1** Upshot is a language for the data-driven aspects of development: managing a program's data, whether in
memory and/or in a database, without having to architect its data structures for efficiency.
Source: L426443, L426464

**LANG-5** Today data held in memory and/or in a database must have its data structures architected carefully and
efficiently, which interferes with maintainability and feature velocity. In Upshot a program defines its data
structures in the way that best serves maintenance and feature development, and the structure that makes it
efficient is provided separately, as optional optimisation metadata (`optimisation.md`).
Source: L426464

**LANG-6** Upshot has two parts: the **feature language**, in which the program, its data structures, its computed
values and its actions are written, and the **optimisation metadata**, kept apart from the program (interpretation:
J426464).
Source: L426464, J426464

## Implementation and targets

**LANG-2** Upshot is implemented in Rust.
Source: L426443

**LANG-3** Upshot targets native interface integration with C and C++: it works with C and C++ code through their
native interfaces (interpretation: J426444).
Source: L426443, J426444

## Out of scope

**LANG-4** The project has no cloud infrastructure and no website of its own. Runtimes that serve a zone as a web
API in the cloud may exist ultimately (RUN-3); they are products of Upshot, not infrastructure of this project, and
out of scope for now (interpretation: J426534).
Source: L426443, L426531, J426534

## Current focus

**LANG-7** For the foreseeable future the work is focused on the feature language with a prototype runtime (RUN-2),
with little to no work on the optimisation language (OPT-1).
Source: L426531
