# Compiler panics and language-limit follow-ups

This document is the intake list for compiler panics and language limits discovered during normal
FPAS work. Resolved behavior belongs in the implementation, regression suite, and current
documentation; broader language proposals belong in their own future notes.

## Entry rule

When development finds a compiler panic or a language limitation that requires a workaround, add an
entry here in the same change. Include the source shape, observed failure or restriction, temporary
workaround, and a concrete later resolution with regression coverage. Do not silently extend the
language or hide the limitation inside a library implementation.

## Open entries

D5-M1/M2 are planned together in
[concrete debug projections](tracepoint-extensions/06-concrete-debug-projections.md).
D5-M3 is the preceding [stable callback names](tracepoint-extensions/05-stable-callback-names.md)
package. These are planned follow-ups, not implemented behavior.

### D5-M1: Erased generic record fields

Status: Open metadata limitation; a debug-only remedy is planned in package 6.
Source syntax and operational runtime layouts remain unchanged.

Source shape: `type Box of T = record Item: T; Other: integer; end record;`
with local `State: Box of integer`. The program compiles and executes normally,
but `--trace 'F::State.Item'` fails during binding because the executable's shared
generic record layout describes `Item` as `dynamic`. `F::State.Other` remains
traceable. The limitation also applies to global generic field paths.

Cause: `lowering/types/layouts.rs` deliberately uses the erased generic template
for one canonical runtime record layout. No concrete instantiation-specific
field type is retained for the trace binder. A runtime integer observed later
cannot certify that a pre-execution selector is a supported scalar.

Workaround: stop at the useful source location and use non-executing
`--print 'State.Item'`, or trace a concretely typed scalar binding already
receiving the field. These inspect the selected stop/current binding, not a
fabricated complete history of the erased field.

Later remedy: design debug-only structural/projection metadata for a concrete
binding's semantic type, separate from the erased runtime record layout. Carry
it through lowering, object encoding/linking and portable image storage; settle
any required format-version change in that separate work. Do not specialize
operational layouts or infer types by executing the program. Regression coverage
must include multiple generic instantiations, aliases, recursive/nested generic
paths, nominal scalar names, global/local/reference roots, atomic binding and
source/object/image round trips. Current D5 VM tests preserve atomic rejection
and positive support for the same record's concrete fields.

### D5-M2: Erased nominal array element names

Status: Open display-metadata limitation; indexed scalar values remain traceable.

For `type Dollars = distinct integer`, a source parameter `Items: array of Dollars`
has executable element type `integer`. The array debug type retains no separate
nominal element display name. `--trace 'F::Items[0]'` therefore reports the
underlying executable name `integer`; it still compares/captures the correct
scalar value. The same applies to erased subrange and payload-free enum names.
Record fields reached through an array keep their own nominal field metadata.

Workaround: inspect the source declaration with source queries, or trace an
existing concretely typed scalar binding/record field where the nominal name is
already retained. Never infer a nominal name by parsing an aggregate display
string or by observing one runtime value.

Later remedy: extend the debug-only structural/projection design in
[D5-M1](#d5-m1-erased-generic-record-fields) to preserve nested nominal element
names through lowering, object/linking and images without changing operational
type erasure. Include nested arrays, aliases, distinct containers and source/
object/image round trips in that follow-up. The current index package does not
silently add a new image format to recover display-only information.

### D5-M3: Anonymous callback names contain AST addresses

Status: Planned in [stable callback names](tracepoint-extensions/05-stable-callback-names.md).

Source shape: a routine containing `function(): integer begin return Total;
end function`. Semantic analysis names the expression `$closure_<key>`, where
`expr_lookup_key` is the AST node address. Lowering publishes that synthetic
name beneath the lexical owner. The key is valid for the immutable compilation
but is not a stable identity across compiler processes.

Observed restriction: an exact anonymous callback name taken from one source
compilation is unreliable as a trace selector for a fresh source compilation.
This also makes stack naming and repeated debugging harder to follow.

Workaround: use a named nested routine, or inspect and trace the same already
compiled portable image without recompiling its source. Do not substitute a
function-variable alias or search by a suffix.

Resolution: keep AST identity keys internal; deterministically number anonymous
literals within their complete lexical owner in source order. Verify separate
compiler processes, exact capture selectors, nested owners, generic routines,
retained defaults and object/image round trips. No source syntax or closure
storage change is required.
