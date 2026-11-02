# CLR mapping and compiler boundary

**Status: target implementation contract; a small subset is implemented.** See
the [first-program milestone](../implementation/first-program.md) for actual
compiler/emitter support. This describes the full language-to-CLR representation.
The separate [MSBuild plan](../build/msbuild.md)
specifies project/solution integration, framework references, restore, and the
compiler's build boundary. IDE and language-service integration are deferred.

## Compilation pipeline

```text
UTF-8 FPAS sources + supplied assembly/reference metadata
  -> syntax trees
  -> binding, overload resolution, generic constraints, nullable flow
  -> typed representation
  -> lowering (closures, unions, contracts, async methods, cleanup)
  -> CIL + metadata + portable PDB + optional XML documentation
  -> managed assembly and normal .NET host files
  -> CLR execution
```

There is no generated C# stage. A library may write the PE/metadata/PDB containers;
direct CIL generation does not require implementing those binary writers from
scratch. The compiler implementation is C#, with the project boundaries defined
in the [repository layout plan](../architecture/project-layout.md). The binary
writer currently uses the built-in System.Reflection.Metadata and ManagedPEBuilder;
that engineering choice is separate from source syntax.

## Identity and representation

| Source construct | CLR representation |
| --- | --- |
| Unit `Game.Geometry` | Public static container type Geometry in namespace Game |
| Free routine | Static method on its unit/program container |
| Public type Point inside that unit | Nested public type; CLR name `Game.Geometry+Point` |
| Primitive alias | The exact System primitive type |
| Transparent alias | No new metadata type |
| Distinct value scalar | Immutable CLR value wrapper with explicit conversions |
| Distinct string | Sealed immutable reference wrapper with a non-null payload |
| Integral subrange | Immutable value wrapper with checked construction and valid default |
| Top-level const / var | Static readonly / mutable static field on the container |
| Language record | Sealed class with read-only properties and an all-fields constructor |
| Record instance / static method | Instance method with self / static method without self |
| Language union | Public abstract closed-in-source base class with generated sealed case classes |
| CLR enum | Enum type and chosen integral underlying type |
| Vector / rectangular / jagged array | Actual corresponding CLR array type |
| Callable shorthand | Matching Func/Action delegate type |
| Compatible named routine/lambda | Target CLR delegate and closure environment as needed |
| Generic type/routine | CLR generic parameters and applicable CLR constraints |
| `ref` / `out` / `in` parameter | Managed byref signature with appropriate parameter metadata/modifiers |
| Nullable value | Nullable<T> |
| Nullable reference | Reference type with nullable metadata annotations |
| Async routine | Task-returning method and compiler-generated async state machine |
| Using / exception handling | CLR protected regions, handlers, filters, and cleanup |
| Requires / ensures | Callee entry/exit Boolean checks and ordinary CLR exceptions |

The same container rule applies to a qualified program name. Visibility is checked
at both source and metadata boundaries. Public nested types in accessible
containers are available to C# and other .NET consumers using ordinary nested
type names. Source-only aliases never become new public types.

## Records and unions

Record properties use their exact source field names and public/private visibility;
constructor parameters use the same names and declaration order. Fields remain
read-only. If any field is private, the all-fields constructor is private and
public factories control external construction. Emit typed equality, Equals, GetHashCode,
and equality operators consistently, using EqualityComparer<T>.Default per field.
Do not generate mutation setters or silently deep-copy fields.

Defaults need not be legal CLR optional-parameter constants. Emit metadata mapping
each defaulted field to a typed static provider method, evaluated in declaration
scope. Publicly constructible records expose the necessary source-hidden providers
to other FPAS assemblies. Their parameters/results and ownership metadata must be
validated before use. A public all-fields CLR constructor still requires its full
argument list; FPAS calls evaluate omitted defaults and then invoke it. Do not
advertise arbitrary computed defaults as optional metadata constants to C# callers.

With updates use typed copy helpers where necessary to preserve private fields
without exposing or rewriting them. The compiler enforces source member access
before selecting helpers; helpers exposed externally cannot update private fields.
They preserve unmentioned values and never reevaluate defaults. Static record
methods emit normal static methods and have no implicit receiver. Private methods
remain private; only public instance members satisfy instance interfaces.

The record metadata contract is version 1 for both generic and nongeneric definitions. Emit the following
attributes from Fpas.Core (the SDK distributes the matching assembly):

- RecordAttribute(int version), applied to the record class with version 1.
- RecordFieldAttribute(int order, string copyMethod, string? defaultProvider),
  applied to each property. Orders are unique, contiguous, and zero-based in
  source field order. The exact property/getter type includes nullable annotations.

A field named X has an instance `<With:X>(FieldType): RecordType` copy helper and,
when defaulted, a static `<Default:X>(): FieldType` provider. Copy helpers have the
field's visibility; default providers have the constructor's visibility. These
names cannot be written as FPAS source identifiers. Readers validate attribute
identity/version, sealed class shape, generic arity, property accessors, field order,
and exact helper/constructor signatures and visibility. Private field types may
remain private; their signatures are inspected only to validate the contract and
are never exposed by public helpers. A reader must reject an incompatible contract
rather than treat arbitrary CLR classes as language records. See the implemented
[record milestone](../implementation/records.md) for the current subset.

Distinct/subrange metadata records nominal identity and the primitive base. Emit
checked construction/conversion methods and read-only Value. Subranges also emit
base-typed LowerBound/UpperBound constants. A zero-initialized subrange must decode
to LowerBound, not an invalid zero; use a private offset encoding as specified in
[data domains](../pascal/language/data-domains.md). This is required for array and
generic defaults as well as explicit construction.
Distinct wrappers expose explicit CLR conversions to/from the base. Subranges
expose an explicit checked base-to-domain conversion and implicit domain-to-base
conversion. A byref signature always retains the wrapper type; it is not an alias
for the payload's address.

Top-level const/var fields share a container type initializer that evaluates
explicit source initializers in written order. Public vars are writable CLR
static fields. No extra global-state scheduler, automatic synchronization, or
cross-task ownership checks are generated.

For unions, emit a `private protected` (CLR FamilyAndAssembly) base constructor,
no externally accessible inheritance hook, an
integral read-only Tag property, and nested sealed case classes named
`<CaseName>Case`. A payload case exposes a static factory named CaseName on the
generic/non-generic union owner. A fieldless case exposes a static property with
that name. Case payload properties preserve source labels. Tag values follow
declaration order and are an assembly implementation detail, not a storage schema.

For example, Shape.Circle(2.0) calls Shape.Circle(double) and produces a
Shape.CircleCase value. Generated symbols participate in collision checks: a
source case/payload name must not collide with reserved generated members such
as Tag, Equals, GetHashCode, or a generated case type name in the same CLR scope.
Union base equality compares case plus payload and handles null consistently.
The base implements IEquatable<Union> and declares abstract Equals(object?),
GetHashCode, and get_Tag slots; sealed cases implement them. Case constructors
are assembly-internal; callers construct through factories/properties.

Cross-assembly matching needs checked case metadata. Emit a compiler-recognizable
union/case annotation from Fpas.Core so consumers do not guess union membership
from arbitrary .NET subclasses. Imported ordinary CLR hierarchies are open for
pattern-coverage purposes. Generic constraints and nullable metadata must also
survive public signatures.

The union metadata ABI is version 1 for both generic and nongeneric definitions:

- Fpas.Core.UnionAttribute(int version) marks the abstract union class with 1.
- Fpas.Core.UnionCaseAttribute(int tag, string name, string factory,
  string[] payloadNames) marks every nested case class. Payload labels are in
  declaration order. A payload case names its static factory; a fieldless case
  names its static getter, get_CaseName, associated with the CaseName property.

Readers verify attribute identity/version, closed constructor accessibility,
abstract base/readonly Tag shape, nested sealed case ownership, exact factory
signatures/parameter names, readonly payload properties/backing fields, nullable
annotations, and unique contiguous tags. Case types have no RecordAttribute;
record with updates do not apply to them. An unknown/incompatible metadata
contract is an error, not a guessed set of subclasses. See the implemented
[union/matching milestone](../implementation/unions-and-matching.md).

## Generic data types

Generic records/unions use ordinary invariant CLR type parameters and constraints.
The nested type name includes its introduced arity, for example ``Box`1`` and ``Tree`1``.
Generated cases are nested in their union, inherit its generic parameters and
constraints, and derive from the corresponding constructed union. A case adds no
new parameters or name arity: C# names Tree<T>.LeafCase, not LeafCase<T>.
Its metadata GenericParam table still contains the inherited parameters.

Fields and member signatures retain type parameters (!n), including constructor,
copy/default helpers, equality, and union factories. A constructed owner is a
TypeSpec, and calls/field access use MemberRefs whose signatures retain !n rather
than substituting the call site's concrete types. Generic methods additionally
use their own !!n slots and MethodSpec instantiations. Combined closure environments
renumber enclosing type and method parameters into their own !n slots; constraints
are re-emitted with the same translation. Nested closure name arity counts only
parameters introduced beyond its containing record.

The existing version-1 record/union attributes need no additional fields for
generics. Readers substitute the actual owner arguments when checking all
property/helper/factory signatures and case bases, retaining nullable annotations.
They verify matching generic arity and all existing readonly/accessibility rules.
See [implemented generic data types](../implementation/generic-types.md).

## Generic routines and closures

Source generic routines emit ordinary CLR generic methods with named parameters,
special/base/interface constraints, and nullable metadata. A source call with
explicit arguments uses a MethodSpec over its MethodDef; an imported call uses a
MethodSpec over the actual generic MemberRef signature. Source declarations do
not specialize into unrelated concrete methods or erase T to object.

Closures from generic methods become nested generic types with the enclosing
method's parameters/constraints. Closure fields, constructors, and invocation
bodies use type parameters (!n), while the creating method supplies its method
parameters (!!n) in the constructed closure type. MemberRefs on constructed
closure types preserve this distinction. GenericParam rows follow the encoded
TypeOrMethodDef owner order, independent of method/type emission order.

Nullable<T> contributes its underlying value type's nullability slot rather than
an additional wrapper slot, including when substitution has not yet resolved a
generic argument. Keep later reference annotations aligned in nested signatures.
See [implemented generic routines](../implementation/generic-routines.md).
[Type argument inference](../implementation/type-argument-inference.md) resolves
the same concrete instantiations before emission. It does not alter generic
signatures, introduce object erasure, or add a runtime dispatch mechanism.

## Small language support library

`Fpas.Core` contains union, record-default, and data-domain metadata annotations,
and only helpers required by emitted language constructs. It does not supply an
implicit type prelude or require Option/Result implementations. Record/union
representation code may be synthesized per assembly. Closure lowering uses normal
CLR objects;
async lowering uses .NET async builders and task types.

The [build integration](../build/msbuild.md) resolves a matching support assembly
and its runtime dependency with the selected compiler/SDK.
It must not contain a second GC, bytecode interpreter, scheduler, universal boxed
FPAS Value type,
HTTP client wrapper catalog, or replacements for .NET collections. Imported API
calls stay statically bound to real referenced types.

## Resource lowering

`using const Name: T := Expression;` and `await using const` acquire a local
resource; they do not introduce a separate statement body. Lower each declaration
to a single initializer followed by protected cleanup around the remainder of
its enclosing lexical scope. Nest cleanup regions in acquisition order so that
disposal runs in reverse order, including when a later acquisition/cleanup fails.
Null resources skip disposal. The binding remains read-only, and value-type
cleanup must operate on the acquired storage rather than a separate boxed copy.

Plain declarations call IDisposable.Dispose; asynchronous declarations await
IAsyncDisposable.DisposeAsync in the async state machine. Preserve this cleanup
on normal completion, return, break/continue, and exceptions leaving the scope.
Try-body resources are disposed before that try's catch/finally bodies run.
Exception filters retain ordinary CLR ordering and may run before cleanup.
An async scope cannot complete its exit before asynchronous cleanup finishes.
See [resource rules](../pascal/language/errors-and-resources.md).

## Contract lowering

Emit requires checks before the user body and ensures checks in a shared normal
exit epilogue after body cleanup. Retain the return expression's value once;
every successful return must pass the epilogue. Body/cleanup exceptions bypass
postconditions. A false condition creates System.InvalidOperationException;
exceptions from evaluating a condition propagate unchanged.

Checks are present in debug and release and live in the callee, including for
delegate, interface, and cross-language calls. They do not depend on a runtime
contract rewriter, Debug.Assert, a second error wrapper, or a custom contract
exception hierarchy. Contract expressions are restricted as specified in
[contracts](../pascal/language/contracts.md).

For async Task<T> methods, emit entry/exit checks inside the state machine.
Ensures binds T and runs after successful async cleanup, before successful Task
completion. Contract failures fault the Task; faults/cancellation bypass ensures.
Plain Task-returning methods without async check their immediate returned object.

## Entry points, runtime, and diagnostics

Emit a generated `<EntryPoint>` method with an accepted CLR entry signature.
A plain program executes its body there; an async program uses a generated
Task-returning body and a synchronous host bridge. Generated method names are
not source identifiers and cannot collide with a user-defined Main routine.
Forward command-line arguments to the program body's `args` binding.

The CLR owns GC, execution, stack unwinding, and task infrastructure. Deployment
and compilation-mode selection are deferred. Running managed FPAS code does not
imply sandboxing or process isolation.

Emit portable PDB sequence points for user statements and source spans for
diagnostics. Closure/async-generated methods require appropriate debug metadata;
having an IL emitter alone does not provide correct stepping. Hidden plumbing
must not masquerade as a source statement. Bind supplied reference metadata
deterministically rather than using runtime reflection enumeration. The MSBuild
plan supplies compile-time references through standard dependency resolution.

## API documentation output

Preserve attached source comments and their locations through declaration binding.
The optional XML documentation pass uses resolved emitted symbol identities and
parameter names; it must not guess IDs from source display text. Apply the
Markdown/section/export rules in [comments and API documentation](../pascal/language/comments.md).
Generate a separate standard .NET XML sidecar with assembly identity, member IDs,
summary, param, typeparam, returns, and remarks elements as applicable. Do not
embed comments into CIL, introduce runtime documentation attributes/helpers, or
copy descriptions onto synthetic helpers and state-machine members.

Only documented externally accessible emitted symbols appear in the public
sidecar. Source-only aliases, private members, and local bindings retain their
source documentation without public XML entries. Bind overload, generic,
nested-type, property, factory, and byref IDs using the standard documentation ID
encoding. XML nested-type separators differ from reflection FullName notation.
Export must be deterministic and correctly escaped. Documentation warnings retain
source spans and follow normal build warning policy; an unwritable requested
output fails the build. The [MSBuild plan](../build/msbuild.md#documentation-output)
defines generation switches, paths, related output files, and incremental inputs.

## Acceptance requirements for a future implementation

1. Build and execute the complete applicable [examples](../pascal/examples.md)
   as separate compilations, including expected error paths.
2. Inspect emitted IL/metadata to establish actual CLR signatures, accessibility,
   generic constraints, nullable annotations, and direct external calls.
3. Reference an FPAS library from C# and an ordinary C# library from FPAS; cover
   records, unions, delegates, overloaded methods, interfaces, and exceptions.
4. Verify checked arithmetic, aliasing, UTF-16 indexing, record field equality,
   enum unnamed values, and null-flow diagnostics with focused cases.
   Reject missing type annotations on local/top-level const/var declarations,
   loop variables, using resources, and const pattern bindings, even when their
   types could be inferred. Named ensures results also require exact type
   annotations. CLR type patterns already state a type before `as`.
5. Exercise async success/failure/cancellation, closure captures, and disposal on
   normal completion, return, exceptions, break, and continue. Verify reverse
   acquisition order, per-iteration lifetimes, nested blocks, try/catch/finally
   ordering, null resources, and later acquisition/cleanup failures. Acquired
   non-null resources must be released once; asynchronous cleanup must finish
   before scope exit completes. Reject resource declarations without const/types
   and outside local statement scopes. Require matching end program/end unit
   source terminators and reject terminal periods or bare program-closing end.
   Confirm nullable values, Try-pattern calls, and user-defined union outcomes
   remain ordinary .NET/data operations with no implicit failure conversion.
6. Reject unsupported metadata/features deliberately and identify them in source
   diagnostics. No dynamic/reflection fallback may hide missing compiler support.
7. Use PDBs to verify source breakpoints and stepping in plain and async functions.
8. Verify contract ordering and one-time execution in debug/release, including
   all normal returns, cleanup ordering, body/cleanup failures, condition failures,
   procedures with ref/out state, block lambdas, generic methods, async routines,
   and calls through CLR interfaces/delegates or a C# consumer. Check rejected
   expressions and mismatched/missing ensures result annotations.
9. Check distinct identity, explicit conversions, subrange limits/extremes,
   valid default-initialized domains, CLR consumer construction, and generic/array
   use without erased validation. Test defaults in written order, skipped explicit
   defaults, private constructors, static factories, with updates preserving private
   state, and global initialization/mutation. Include cross-assembly metadata cases.
10. Verify comment attachment/extraction and structured documentation diagnostics.
    Parse XML output, compare documentation IDs and parameter names with emitted
    signatures, and check escaping, generic/byref/nested symbols, deterministic
    output, and C# reference-sidecar propagation with generation enabled/disabled.

The first-program milestone covers only its documented subset of these checks.
The complete language and acceptance list remain implementation work.
