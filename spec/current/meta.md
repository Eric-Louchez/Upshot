# Metaprogramming — current specification

Programs whose structure is itself computed.

Derived from the log; where it disagrees with the log, the log wins (spec/README.md).

**META-1** The feature language supports metaprogramming: the structure of computed values, and an API itself (which
values and actions it exposes), can be computed from other values. For example, the API exposed to or used in a
session can be computed from the user's role. An API whose shape is computed from a configuration value is fixed for
the session (interpretation: J427100, confirmed by L427101).
Source: L427098, J427100, L427101
