# Actions — current specification

How a program acts on its data. The user describes this model as their current view, so it is provisional
(J426468).

Derived from the log; where it disagrees with the log, the log wins (spec/README.md).

**ACT-1** An action acts on the data: it is a data transformation of its inputs and the current dataset into
values that become connected to data sources of the dataset, or to computed data with a two-way computed path to
data sources (MODEL-4).
Source: L426464

**ACT-2** An action's results are written only to data sources, or to bidirectional computed values that carry the
write back to their sources (MODEL-8); an action never writes a one-way computed value (interpretation: J426468).
It can instead convert a computed value into a source, or a source into a computed value (MODEL-9).
Source: L426464, J426468, L427008

**ACT-3** Actions are atomic: an action's changes all take effect together or not at all. An action that would
violate a validation constraint, or that triggers a cyclic dependency (MODEL-10), fails and is rolled back. How an action can read computed values that depend on its own
changes so far, before it completes, is not yet decided (`plan/README.md`).
Source: L427008, J427008, L428317

**ACT-4** Functions (actions and computation functions) may contain exception handling: a failed function triggers
an exception-handling function. A failed action's own changes still never take effect; the handler runs as a
separate action of its own, whose changes can take effect. A failure with no handler passes to the caller
(interpretation: J427076, confirmed by L427098).
Source: L427076, J427076, L427098

**ACT-5** Actions are like computation functions temporarily connected to the source values they modify: they can
use the same structures and annotations as computation functions (interpretation: J428001).
Source: L428000, J428001

**ACT-6** Actions and computation functions can each be written in either of two forms: as a sequence of operations
some of which write to targets, or as one definition per target value (possibly through a helper that computes
several values as a tuple, each target taking its entry). An action written per target defines the value of every
target it writes; a computation function written as a sequence can produce several computed values
(interpretation: J428060).
Source: L428060, J428060
