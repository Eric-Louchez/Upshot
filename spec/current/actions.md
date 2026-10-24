# Actions — current specification

How a program acts on its data. The user describes this model as their current view, so it is provisional
(J426468).

Derived from the log; where it disagrees with the log, the log wins (spec/README.md).

**ACT-1** An action acts on the data: it is a data transformation of its inputs and the current dataset into
values that become connected to data sources of the dataset, or to computed data with a two-way computed path to
data sources (MODEL-4).
Source: L426464

**ACT-2** An action's results are written only to data sources, or to bidirectional computed values that carry the
write back to their sources; an action never writes a one-way computed value (interpretation: J426468).
Source: L426464, J426468
