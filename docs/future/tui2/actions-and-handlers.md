# Std.Tui2 actions and handlers

The earlier named-handler design in this document has been superseded.

The authoritative future contract is now [events-and-actions.md](events-and-actions.md). It defines
Pascal-style event assignment, capturing handlers, bound record methods, reusable `TuiAction`
handles, deterministic single-handler execution, and the absence of a general publish/subscribe
bus.

This forwarding page remains temporarily because other Tui2 phase documents still link to its old
name. Remove it when those phase references are reconciled during event implementation.
