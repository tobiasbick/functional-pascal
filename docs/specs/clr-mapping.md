# CLR mapping and compiler boundary

This language-to-CLR contract describes implemented behavior. See
the [compiler overview](../architecture/compiler.md) and
[acceptance coverage](../../tests/coverage.md) for actual
compiler/emitter support and verification.
The [MSBuild contract](../build/msbuild.md)
specifies project/solution integration, framework references, restore, and the
compiler's build boundary. The [language server](../pascal/tools/language-server.md)
owns implemented source services; further IDE integration belongs in
[editor gaps](../future/editor-gap.md).

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
in the [repository layout](../architecture/project-layout.md). The binary
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
| typeof(T) | Type token loaded with ldtoken and resolved by System.Type.GetTypeFromHandle |
| Async routine | Task-returning method and compiler-generated async state machine |
| Using / exception handling | CLR protected regions, handlers, filters, and cleanup |
| Requires / ensures | Callee entry/exit Boolean checks and ordinary CLR exceptions |

The same container rule applies to a qualified program name. Visibility is checked
at both source and metadata boundaries. Public nested types in accessible
containers are available to C# and other .NET consumers using ordinary nested
type names. Source-only aliases never become new public types.

Executable PE images set ExecutableImage without the Dll characteristic and have
a real CLR entry-point token, including the async entry bridge. Libraries set
Dll and have no entry point. The managed implementation filename remains `.dll`.
This distinction also lets Native AOT identify an executable input module.

Optional strong-name signing writes the RSA public key into Assembly metadata,
sets PublicKey and reserves the CLR signature directory. Full signing uses the
ECMA-335 SHA-1/RSA-PKCS1 strong-name format and sets StrongNameSigned. PublicSign
sets that flag with a zero signature; DelaySign leaves it clear with a reserved
signature. Unsigned images have neither public key nor signature reservation.
The content MVID covers code/metadata/symbol linkage before signing, with the MVID
and signature slots zeroed. Signing does not change source type/member mappings.
The [publishing contract](../build/publishing.md) owns keys, SDK properties and
delivery signatures, which are separate from CLR assembly identity.

Public aliases retain their target through a private static, nongeneric
`<Alias:Name>(): Target` method on the unit container, marked with
Fpas.Core.AliasAttribute(int version, string name), version 1. Its return signature
preserves nullable annotations recursively. This method is metadata-only and
throws NotSupportedException if invoked through reflection. It is not a public
API or a new type. Importers validate the version, exact helper name, private
static visibility, zero parameters, and publicly accessible target signature.
Private aliases need no emitted contract.

Rectangular array signatures use rank 2..32, no fixed sizes and one explicit zero
lower bound per dimension, matching ordinary C# signatures. Member references
retain that form so actual .NET methods resolve. Other encoded bounds/sizes,
including omitted rectangular bounds, are rejected as unsupported during import
rather than silently rewritten. These signature bounds do not constrain an
array instance's runtime lower bounds.

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
rather than treat arbitrary CLR classes as language records. See
[record semantics](../pascal/language/types/records.md).

Distinct/subrange metadata records nominal identity and the primitive base. Emit
checked construction/conversion methods and read-only Value. Subranges also emit
base-typed LowerBound/UpperBound constants. A zero-initialized subrange must decode
to LowerBound, not an invalid zero; use a private offset encoding as specified in
[subrange defaults](../pascal/language/types/subranges.md#clr-default-value-invariant). This is required for array and
generic defaults as well as explicit construction.
The version 1 contract uses Fpas.Core.DomainAttribute(int version,
string primitiveType, string? lowerBound, string? upperBound), where primitiveType
is the full System type name and bounds are invariant decimal integer strings.
Distinct types have two null bounds; subranges have both bounds. The sealed,
nongeneric wrapper derives directly from System.ValueType, or System.Object for
distinct strings. Its single private readonly instance field has the primitive
type for distinct values and UInt64 for subranges. Subrange offsets use modular
64-bit subtraction/addition, followed by unchecked narrowing to the base.
Importers validate the attribute identity/version, base, storage, read-only Value,
public constructor and conversion signatures, and exact bound constants. Invalid
contracts fail before emission. Value equality/hashing uses
EqualityComparer<Base>.Default; numeric/char ordered operators use numeric
comparison, and string ordered operators use String.CompareOrdinal.
Distinct wrappers expose explicit CLR conversions to/from the base. Subranges
expose an explicit checked base-to-domain conversion and implicit domain-to-base
conversion. A byref signature always retains the wrapper type; it is not an alias
for the payload's address.

Top-level const/var fields share a container type initializer that evaluates
explicit source initializers in written order. Public vars are writable CLR
static fields; const fields are static InitOnly fields, never Literal fields.
Containers with globals have a private static .cctor marked SpecialName and
RTSpecialName, without BeforeFieldInit. Active static calls/field access trigger
CLR initialization; metadata inspection does not. Forward and cyclic reads can
observe default storage before an initializer runs. CLR initialization failures
and concurrency semantics are preserved. Field signatures carry the same nullable
annotations as routine signatures. Source readonly fields are not compile-time
constants for defaults, constant patterns, scalar bounds, or contract conditions.
No extra global-state scheduler, automatic synchronization, or
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
contract is an error, not a guessed set of subclasses. See
[union semantics](../pascal/language/types/unions.md).

## Source interfaces

A source interface emits a nested Interface/Abstract CLR type under its
unit/program container with the ordinary source visibility and generic arity.
It has no base class, instance fields, or constructors. Each declared signature
emits a public abstract virtual newslot instance MethodDef with no CIL body.
Getter-only declarations emit Property/getter/MethodSemantics metadata. Bases
emit InterfaceImpl entries, including constructed source or imported interfaces.

Generic interfaces are invariant. Emit type/method constraints, nullable metadata,
parameter labels/defaults/params and ref/out/in modes on their ordinary CLR
targets. An in parameter requires the InAttribute required signature modifier
as well as the existing parameter flags and readonly annotation. A reference
constraint inferred from a base constraint does not add an explicit class flag;
this preserves C# generic implementation compatibility.

Records use the same validated requirements and MethodImpl dispatch mapping for
source and imported interfaces. Preserve each distinct declaring slot even when
one record method satisfies several slots. Generic MethodImpl declarations use
MemberRef signatures, not MethodSpec instantiations. Inherited diamonds deduplicate
the same slot; incompatible requirements and ambiguous access produce diagnostics.
Constraints and nullable safety remain checked across assembly boundaries.
External consumers read ordinary CLR metadata; no additional Core annotation is
needed. XML documentation uses normal nested type, method, and property IDs.
See [interface semantics](../pascal/interop/interfaces.md).

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
See [implemented generic data types](../pascal/language/types/generics.md).

## Lifted nullable operators

Lift supported non-nullable value operators through Nullable<T>, retaining their
ordinary result types, numeric promotion, checked behavior, and CLR dispatch.
Bind each operand to a compiler-generated local in written order before testing
HasValue. Execute the underlying operation only when all operands are present.
Wrap value results in Nullable<Result>, or produce its empty value. Equality
compares presence when an operand is absent; relational comparisons then return
false. Comparisons always return bool. No nullable Boolean logical operators or
new reference-type operator rules are introduced.

Reuse ordinary bound sequences, conversions, null tests, and conditional
expressions. The same lowering preserves order and operand storage across await,
and exposes operand effects to definite-assignment and contract validation.
No runtime operator wrapper or reflection dispatch is needed. See the complete
[nullable operator contract](../pascal/language/types/nullable-types.md#lifted-value-operators).

## Loop lowering

Repeat/until uses an iteration body followed by a Boolean test, with continue
targeting the test after lexical cleanup and break bypassing it. Counting loops
cache both typed bounds once, compare with correct signedness, and guard the
endpoint before checked stepping/narrowing or subrange construction. Synchronous
CIL and async state-machine lowering use the same bound operations. Per-iteration
counter captures preserve the source value, including subrange wrapper identity.
See [implemented loops](../pascal/language/control-flow/loops.md).

## Generic routines and closures

Source generic routines emit ordinary CLR generic methods with named parameters,
special/base/interface constraints, and nullable metadata. A source call with
explicit arguments uses a MethodSpec over its MethodDef; an imported call uses a
MethodSpec over the actual generic MemberRef signature. Source declarations do
not specialize into unrelated concrete methods or erase T to object.

Imported static abstract/virtual interface members retain their constructed
interface MemberRef and the constrained implementing type parameter. Calls emit
`constrained. T` followed by `call`; method-group delegates use `constrained. T`
followed by `ldftn`. Preserve that parameter through inferred generic calls,
closures, record methods, and async state machines. Operator selection uses the
declared interface signatures and checked variants; a direct call on an interface
without an implementing type parameter is rejected. See the CLR's
[static interface dispatch rules](https://github.com/dotnet/runtime/blob/main/docs/design/specs/Ecma-335-Augments.md).

Closures from generic methods become nested generic types with the enclosing
method's parameters/constraints. Closure fields, constructors, and invocation
bodies use type parameters (!n), while the creating method supplies its method
parameters (!!n) in the constructed closure type. MemberRefs on constructed
closure types preserve this distinction. GenericParam rows follow the encoded
TypeOrMethodDef owner order, independent of method/type emission order.

Named nested routines emit instance methods on a private generated closure type
for their declaring block. They retain source names and ordinary overload/byref/
optional/params metadata. Enclosing generic parameters belong to the closure;
each method's own parameters remain !!n in its method signature. An async state
machine carries both sets as parameters of its generated type.
Environment construction captures immutable values and shared mutable cells at
the call/delegate conversion site. Recursive/sibling calls use the current
receiver. Validate captured-binding initialization before construction; do not
replace uninitialized values with defaults. Local declarations emit no executing
statement, public API, or public XML entry. See
[nested routines](../pascal/language/functions/nested-routines.md) for the implemented scope.

Private closure environments use source-derived capture field names, with
deterministic numeric suffixes for collisions in the same environment. All
generated references use the same selected field identity. This supports
ordinary debugger expansion without changing capture types, shared-cell
identity, normal execution, or public source declarations. Mutable debug views
read the shared cell's current value rather than a copied local snapshot.
Portable PDB source-storage entries identify direct captures and compiler cells
by field/local identity, with their original source names. Passive readers use
that mapping rather than unwrapping user StrongBox values by type-name heuristics.
See the [storage format](../architecture/debug-information.md#passive-source-storage).

Private async state machines distinguish source-hoisted fields from synthesized
temporaries. Source display names are unique within the machine, including
parameter/local collisions; numeric suffixes disambiguate bindings without
changing their storage or public names. Portable PDB hoisted scopes use emitted
lexical block intervals and preserve slot indexing for unavailable/internal
entries. Generated frame/type names remain CLR details, not new FPAS syntax.

Nullable<T> contributes its underlying value type's nullability slot rather than
an additional wrapper slot, including when substitution has not yet resolved a
generic argument. Keep later reference annotations aligned in nested signatures.
See [implemented generic routines](../pascal/language/functions/generic-routines.md).
[Type argument inference](../pascal/language/functions/type-inference.md) resolves
the same concrete instantiations before emission. It does not alter generic
signatures, introduce object erasure, or add a runtime dispatch mechanism.

## Debugger metadata

Module identity is derived from serialized PE content with the MVID slot zeroed
during hashing. It identifies emitted code/metadata and symbol linkage, including
reference-driven changes, rather than only source text. Repeated identical
emission remains byte-identical. See
[portable identity](../architecture/debug-information.md#source-and-method-identity).

Generated debug presentation uses standard .NET DebuggerDisplay,
DebuggerTypeProxy, DebuggerBrowsable and generated-code/stepping attributes.
Record/case proxies are private nested `<DebugView>` types with typed getters;
they preserve generic arguments and do not change the record/union metadata ABI.
Private backing fields are hidden from ordinary debugger expansion. Domains
display their decoded Value. Source-authored debugger display/proxy annotations
override the generated defaults. These attributes are not FPAS reflection
markers and do not require an F# runtime. See
[debug information](../architecture/debug-information.md#debugger-attributes-and-value-views)
for the emitted attributes and stepping boundaries.

The own debugger's passive source views read the versioned record, union-case
and domain metadata and actual stored fields. They do not invoke generated
proxies, getters, Tag, display attributes or user methods. Raw field expansion
remains available independently of an optional source-presentation error. See
the [debugger value contract](../architecture/debugger-core.md#passive-stored-values).

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

## Source attributes

Source custom attributes emit actual constructor MemberRefs and ECMA-335 blobs on
their mapped type/method/property targets. Record field annotations apply only to
the property. Preserve constant widths, boxed payload types, array/null encodings,
and qualified nested/generic type names. Serializable and supported MethodImpl
options instead set their actual CLR flags. No attribute constructor executes
during compilation. See [source attribute mapping](../pascal/interop/attributes.md).

## Event accessors

subscribe/unsubscribe selects the actual Event/MethodSemantics add/remove handle
from supplied metadata, checks that accessor, and emits an ordinary method call.
Receiver storage and handler evaluate once in written order before invocation.
Preserve static/virtual/interface dispatch, generic substitutions, nullable
annotations, and writable struct/constrained receiver storage. Existing async
lowering preserves reference receivers/handlers across suspension and rejects
managed receiver references live across await. No delegate field read, event
raising operation, reflection dispatch, or automatic removal is generated.
See [event binding and evaluation](../pascal/interop/events.md).

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
See [resource rules](../pascal/language/error-handling/README.md).

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
[contracts](../pascal/language/functions/contracts.md).

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
contract supplies compile-time references through standard dependency resolution.

## API documentation output

Preserve attached source comments and their locations through declaration binding.
The optional XML documentation pass uses resolved emitted symbol identities and
parameter names; it must not guess IDs from source display text. Apply the
Markdown/section/export rules in [comments and API documentation](../pascal/language/basics/comments.md).
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
output fails the build. The [MSBuild contract](../build/msbuild.md#documentation-output)
defines generation switches, paths, related output files, and incremental inputs.

## Verification

The [coverage map](../../tests/coverage.md) links this contract to compiler,
metadata, CLR execution, mixed-project and debugger tests, including rejection
cases. Complete [examples](../pascal/examples/README.md) are compiled and executed
from Markdown. Only open work belongs in [language gaps](../future/language-gaps.md).
