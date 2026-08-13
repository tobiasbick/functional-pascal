# Consciously deferred scope

These exclusions are deliberate boundaries of explicit aggregate construction.
They are not partially implemented by this package and must not be inferred
from the discovery or construction protocol.

## AC-D01: Implicit stale-handle switching

An old payload-child `variablesReference` never selects a different variant.
After mutation, clients must request current variables again.

Re-entry requires a versioned stable-object identity model that can distinguish
a deliberate variant switch from an expired reference without guessing.

## AC-D02: Partial or incremental construction

The debugger never commits a variant with missing fields and never exposes a
builder object as a live program value.

Re-entry requires a separate detached-value builder protocol with explicit
lifetime, cancellation, limits, and final atomic commit semantics.

## AC-D03: Virtual inactive Variables children

Inactive variants are not advertised as writable synthetic children in the
standard DAP Variables tree. Discovery and the explicit command are the only
new selection surfaces.

Re-entry requires stable synthetic-reference semantics across refreshes and a
clear standard-client interaction that cannot be confused with live children.

## AC-D04: Defaults, omitted fields, and hidden-field synthesis

Every declared payload field must be supplied. The debugger does not execute
source constructor defaults or invent values for metadata that is not portable.

Re-entry requires executable metadata and source-equivalent rules proving that
debugger construction matches ordinary FPAS construction exactly.

## AC-D05: Missing outer storage

Construction may initialize one complete mutable root, but it does not allocate
an uninitialized outer record/array/wrapper descendant, create a missing capture
cell, create an absent parameter, or suppress a later source initializer.

This remains owned by central backlog package `DBG-D02`.

## AC-D06: Identity-bearing or unsafe payload values

New closures, task-bound functions, `Dynamic` callable endpoints, task handles,
capture cells, and opaque resources are not constructed as fields merely because
a variant operation exists.

This remains owned by central backlog package `DBG-D03` and by the implemented
debug-evaluation safety policy.

## AC-D07: Language and standard DAP changes

No FPAS constructor syntax changes and no reinterpretation of standard DAP
`setVariable` or `setExpression` is included.

Re-entry for a language change requires explicit user agreement. A standard DAP
change requires a portable contract accepted by ordinary DAP clients.
