# Constraints — current specification

Validation constraints: conditions the data must always meet, and what happens to an action that would break one.

Derived from the log; where it disagrees with the log, the log wins (spec/README.md).

**CONS-1** The feature language lets a program declare validation constraints: conditions over the data, computed
like computed values (they may use computed values and arbitrary computation functions), that must always hold
(interpretation: J427008).
Source: L427008, J427008

**CONS-2** An action that would make a constraint false is rejected: none of its changes take effect (ACT-3;
interpretation: J427008). The rejection is a failure that an exception-handling function can handle (ACT-4).
Source: L427008, J427008, L427076
