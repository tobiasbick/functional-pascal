# AP38: Routine declaration visibility

Status: Complete; scope-wide signatures and checked initialization implemented.
Effort: medium to large. Completion is tracked in
the [central plan](../README.md).

## Addressed finding

Free routines previously required preceding declarations, while type/member
signatures were scope-wide. Record bodies and defaults also resolved helpers at
the record's source position. AP38 now gives named routines one lexical
visibility rule and checks value availability separately; moving a helper no
longer changes forward-reference availability.

## Selected contract

Named routines are known throughout their lexical declaration scope. Values
may be used only after their initialization. This is the single rule for free
functions/procedures, record methods, and named nested helpers.

- Collect named routine signatures before checking bodies. Calls may precede
  declarations, and sibling routines may be mutually recursive. No `forward`
  declaration or additional call syntax is introduced.
- Resolve names lexically: the nearest scope wins. A named inner routine
  shadows an outer name throughout its scope; duplicate declarations in one
  scope remain errors. Source order must not change the selected declaration.
- Preserve source-order initialization of `var` and runtime-initialized
  `const`. Do not reorder initializers by dependency or introduce lazy
  initialization. A callable stored in a binding, including an anonymous
  function, is a value and becomes available through ordinary initialization.
- Routine bodies may refer to later bindings in their lexical environment.
  At a call, the bindings that may be accessed must already be initialized,
  including dependencies reached through other routines or callbacks.
  Ordinary executable expressions cannot use a binding before initialization.
- Initializers may call later-declared routines when the compiler can prove
  that their dependencies are available. Reject unsafe or unprovable uses at
  compile time, including indirect accesses and premature closure captures.
  Closure creation must not copy an uninitialized value.
- Resolve record field defaults in the record's lexical declaration scope;
  evaluate and check their dependencies at construction. Preserve unit
  dependency initialization order and existing public/private boundaries.

Initialization analysis is internal compiler bookkeeping, not a public effect
system or a revival of AP25 purity. Access to initialized mutable bindings and
ordinary effects retain their existing rules. Diagnostics identify the unsafe
dependency and give a concrete repair, such as passing initialized data as an
argument or moving the use after the dependency's initialization.

The decision and regression examples are recorded in
[AP38.1](01-visibility-decision.md). AP38 replaces the routine/value source-order
visibility restrictions documented by AP11/AP24 without changing their
completed status or type/member signature rules.

## Open decisions

None. AP38.2 delivers the conservative initialization proof; inability to prove
a startup use safe is a compile-time error.

## Dependencies and work packages

AP11/AP24 supply forward type/member handling. AP38.1 precedes AP38.2.
No dependency on AP12 or a new callable-expression form exists.

- [x] [AP38.1: Visibility decision](01-visibility-decision.md)
- [x] [AP38.2: Routine signatures](02-routine-signatures.md)

## Acceptance

One lexical signature-visibility rule explains methods, free routines, and
nested helpers. Mutual recursion needs no structural workaround. Initializers
retain their order and cannot expose uninitialized values through direct calls,
callbacks, closure captures, or record defaults. Compiler and editor agree on
declaration identity, including shadowing and independently compiled units.
