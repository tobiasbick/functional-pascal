# FPAS 2 language directions

Status: proposals retained for later evaluation; no implementation is scheduled.
These are design questions, not approved language changes or the current spec.
FPAS 1 stabilization is complete. Selected FPAS 2 topics use the
[AP workflow and templates](improve-syntax/README.md); selecting a topic still
requires an explicit decision before implementation.

## Boundary

Keep functional Pascal with selected Ada conventions, predictable authoring and
few special cases. FPAS 1 should not gain a new effects system, ownership type
system or broad standard-library redesign to address these proposals. Prefer a
single replacement contract over parallel old/new APIs if a proposal is selected.

Preserve the deliberately different roles of positional/named complete calls,
value/InPlace operations, Result propagation/local handling, and lexical scopes
versus dynamic task groups. Similar expressiveness alone is not grounds for removal.

## 1. Typed operational errors

Channel closure, timeout and cancellation are currently different strings inside
`Result of (T, string)`; see [task APIs](../pascal/std/concurrency/task.md).
Programs that distinguish them must compare diagnostic text.

Consider typed error variants such as Closed, Timeout and Cancelled, with
human-readable context kept separately. Callers could use exhaustive case
matching without depending on wording. Evaluate the scope across task, network
and host APIs together; do not add a second parallel family of wrappers.

Open questions: shared versus unit-specific error types, context payloads,
translation across API boundaries and compatibility with generic Result/try.

## 2. Uniform first-class operations

[Ordinary generic routines](../pascal/language/functions/generic-routines.md)
can specialize from a receiving callable signature. Some polymorphic intrinsics,
such as Math.Abs, cannot be used directly that way. Native receiver operations
also require wrapper functions when used as callbacks.

Consider making declared signatures sufficient to obtain callable values,
independent of whether implementation uses source code or an intrinsic. Preserve
static parameter labels, receiver capture by value, var modes and task effects.
Arbitrary callable-expression chaining remains deferred and is outside this topic.

Open questions: which native operations can be bound, variadic operations,
generic specialization and the cost of consistent compiler/editor support.

## 3. Separate equality from ordering constraints

A concrete record with equality-capable fields may be a dictionary key, but a
generic key currently needs Comparable, which excludes records. See
[dictionaries](../pascal/language/types/dictionaries.md) and
[constraints](../pascal/language/types/generics.md#built-in-constraints).

Consider a distinct equality capability so generic code can accept every key
type accepted by equivalent concrete code. Keep ordering as the requirement for
sorting. Determine whether collection equality belongs in the same design;
do not automatically add interfaces, arbitrary constraints or user-defined operators.

Open questions: the smallest constraint vocabulary, structural propagation,
recursive types, and effects on existing dictionary and comparison checking.

## 4. Task completion and result retrieval

`Wait` consumes a result once across all aliases of a handle, while WaitAll,
WaitAny and TaskCase only observe completion. A copied handle therefore does
not provide an independently readable result; see
[waiting](../pascal/std/concurrency/task.md#waiting-for-one-task).

Consider whether completed results should be readable repeatedly, keeping the
existing ownership and cleanup boundaries. Compare the simpler caller model
against result retention, shared-resource payloads, failure propagation and
supervised logical tasks. Do not assume a new linear/ownership type system is
necessary. Until a replacement is agreed, retrieve once and pass the value on.

## Other deferred scope

Explicit-width numeric types and Decimal need a separate coherent numeric
design, migration and buffer/API assessment. They are not FPAS 1 prerequisites.
A purity/effects system was rejected and is not reopened here. Reconsideration
would require a new decision and a demonstrated benefit. FPAS currently remains
function-oriented with explicit binding mutation, rather than a guarantee that
every function is pure.

## Evaluation before implementation

For each selected topic, first record concrete failing or awkward examples,
the smallest replacement contract, migration scope, compiler/runtime/tooling
cost and tests that establish the improvement. Resolve the open decisions with
the user before creating implementation work. This document is not a second
package-completion index.
