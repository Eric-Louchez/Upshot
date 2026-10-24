# Runtime — current specification

Runtimes: what runs a data zone, how a zone is given one, and which runtimes exist.

Derived from the log; where it disagrees with the log, the log wins (spec/README.md).

**RUN-1** Each data zone (ZONE-1) is assigned to a runtime. The assignment is configuration, part of the
optimisation language (OPT-1), not of the feature language.
Source: L426531

**RUN-2** For now the only runtime is a **prototype runtime implemented in Rust**.
Source: L426531

**RUN-3** Ultimately a variety of runtimes could exist, including runtimes on the cloud serving zones as web APIs;
none but the prototype is in scope now (LANG-4, LANG-7).
Source: L426531
