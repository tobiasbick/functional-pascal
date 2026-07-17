# Std.Tui2 application state

The earlier requirement for application-private unit state and fixed named-handler signatures has
been superseded.

Capturing closures are now the normal application-state mechanism. Private unit state remains
legal, and integer tags remain optional association keys, but neither is required to connect a
handler to its state.

The authoritative contracts are:

- [capturing closures](../../pascal/language/functions/closures.md) for environment ownership, mutation, lifetime, and
  task transfer;
- [record events](../../pascal/language/types/record-events.md) and
  [bound record methods](../../pascal/language/types/record-methods.md#bound-methods-as-values) for the
  language-level handler model;
- [Tui2 events and actions](events-and-actions.md) for concrete signatures and registry ownership.

This forwarding page remains temporarily because the implementation phase plan still links to its
old contract. Remove it when that phase is reconciled.
