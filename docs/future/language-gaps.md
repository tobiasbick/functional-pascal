# Language gaps and deferred extensions

**Priority: finish the agreed language and MSBuild baseline before implementing
extensions.** This document records confirmed implementation gaps and a deferred
interface proposal. It does not expand the baseline or claim that its remaining
requirements have all been verified.

The scope is defined by the [language overview](../pascal/README.md),
[language topics](../pascal/README.md#reading-order),
[syntax grammar](../specs/syntax.md), [CLR mapping](../specs/clr-mapping.md), and
[MSBuild plan](../build/msbuild.md). The
[implementation documents](../README.md) describe current support;
[dogfooding gaps](dogfooding-gaps.md) describe practical maintenance limits.

## Finish the existing baseline first

[Imported CLR event subscription/removal](../implementation/events.md) is now
implemented and verified through compiler/metadata tests, mixed C#/FPAS projects,
and an executable collection-events example.
[Runtime type expressions](../implementation/typeof.md) now implement typeof(T)
with CLR type tokens, concrete/generic identity, and cross-language verification.

The following are implementation tasks for already specified behavior, in the
proposed execution order. Implement one focused step at a time without adding
syntax beyond the existing specification.

| Order | Required behavior | Current gap and specification |
| --- | --- | --- |
| 1 | Source attributes on types, routines, and record fields | Attribute syntax, binding, usage checks, and custom-attribute emission are missing. Follow [interop attributes](../pascal/interop/README.md#attributes), including property mapping for record fields and documentation attachment. Compiler-generated attributes already exist; typeof(T) now supplies type-valued arguments. |
| 2 | Fully qualified nested CLR generic type paths | Types returned by CLR APIs retain their identity, but source annotations cannot put generic arguments on every enclosing/nested name segment. Follow [generic type syntax](../pascal/language/functions.md#generic-routines-and-constraints) and the [grammar](../specs/syntax.md). |
| 3 | Operators through constrained imported generic interfaces | Instance constraints work, but generic-math/static-interface dispatch is not implemented. Follow [generic constraints](../pascal/language/functions.md#generic-routines-and-constraints). This consumes existing CLR contracts; it does not add source interfaces, source operators, or static interface implementations on records. |

After these known gaps, compare every baseline rule and complete language example
with compiler behavior. Record any additional demonstrated omissions here before
calling the baseline complete. Resolve stale implementation-status prose as each
feature lands. Validate the agreed .fpasproj/.slnx build, dependency, diagnostic,
incremental, documentation, and cleanup contracts; the MSBuild milestone is not
by itself proof that the entire build plan is implemented.

Numeric lifted nullable arithmetic is recorded as unsupported in the
[scalar milestone](../implementation/scalar-types.md). The baseline defines
nullable storage, flow, validation, and coalescing, but does not give a complete
lifted-operator contract. Do not silently add a new operator/null behavior while
closing the explicitly specified gaps. Revisit only if the baseline comparison
demonstrates an existing requirement or the user chooses to specify it.

## Deferred: declare FPAS interfaces

**Status: future proposal; implement only after completing the original plan.**
The baseline explicitly excludes authored interfaces. Consuming CLR interfaces
and implementing them with records is already supported and remains part of the
baseline. The proposal below is separate from that existing support. Its new
syntax must be confirmed before implementation.

### Purpose

An interface groups related operations behind a common nominal CLR contract.
Different FPAS records or external .NET implementations can satisfy it. Use
ordinary functions/delegates for single operations; a record of delegates can
group callbacks without adding interface declarations. Own interfaces are an
optional modeling tool and interoperability feature, not a prerequisite for
the game or Mandelbrot.

### Proposed syntax

The following is a future design example, not an executable current example:

```pascal
unit Examples.Contracts;

uses System as Sys, System.Threading.Tasks as Tasks;

public type ISource<T> = interface
  Name: string;
  function Read(): T;
end interface;

public type IRefreshableSource<T> = interface extends ISource<T>, Sys.IDisposable
  function RefreshAsync(): Tasks.Task;
  function Map<U>(Transform: function(T): U): U;
end interface;

public type ConstantSource<T> = record implements ISource<T>
  Name: string;
  Value: T;

  function Read(): T;
  begin
    return self.Value;
  end function;
end record;

end unit;
```

Interface type and method constraints use the existing where placement:

```pascal
public type IFactory<T> =
where T: class;
interface
  function Create(): T;
  function Echo<U>(Value: U): U;
  where U: class;
end interface;
```

### Proposed rules

- A declaration uses type Name, optional generic parameters/where clauses, and
  interface ... end interface;. Public declarations belong in units; default
  visibility and accessibility follow ordinary source type rules.
- Members are function/procedure signatures ending in a semicolon, followed by
  any method where clauses. There is no begin or end function/procedure because
  no implementation body is declared. All parameter/result types are explicit.
- Name: T; declares a getter-only property, not instance storage. It has no
  initializer. A matching public record field property can implement it.
- Members are public abstract instance members. Explicit public is permitted;
  private/static members, bodies, constructors, events, setters, and indexers are
  outside this proposal. Empty interfaces are permitted.
- Optional extends lists one or more non-nullable source/imported interfaces.
  It cannot list classes. Check accessibility, duplicate bases, and inheritance
  cycles. Substitute generic arguments throughout inherited signatures. A diamond
  reaching the same base member does not require duplicate implementations.
- Different inherited slots with compatible signatures may share one record
  implementation. Reject incompatible requirements, ambiguous calls, and
  declaration/accessor collisions; do not choose a base by declaration order.
  Preserve existing checks for unsupported inherited requirements and imported
  default-interface implementations.
- Generic interfaces are invariant initially. Generic method constraints,
  overloads, optional/params parameters, ref/out/in modes, and nullability use
  the existing rules. Method matching checks CLR identity, generic arity,
  parameter modes, nullable safety, and compatible generic constraints.
- Record implements accepts source as well as imported interfaces. Missing
  implementations are errors. There is no structural conversion merely because
  a record happens to have matching members, and no explicit-interface syntax.
- Task/Task<T>-returning signatures express asynchronous APIs. async belongs
  to an implementing function body, not to an abstract interface signature.
- Contracts remain on implementations. Interface-level requires/ensures and
  inherited contract composition are not introduced. Existing implementation
  checks still execute through interface calls and across .NET languages.

### CLR representation and implementation outline

Emit an actual nested CLR interface under the unit/program container, using the
existing source type identity/visibility rules. Emit abstract virtual instance
methods without CIL bodies and real getter-only Property/getter metadata. Emit
base InterfaceImpl entries, generic parameters/constraints, parameter modes,
nullable annotations, and normal XML API documentation identities.

Records use the existing interface validation, virtual/final methods, and
MethodImpl machinery. Calls use normal interface dispatch, including constrained
calls when appropriate. No generated C#, runtime reflection dispatch, extra VM,
or interface-specific Fpas.Core wrapper is needed. External FPAS readers consume
ordinary CLR interface metadata rather than a new compiler-private contract.

Implementation steps after the original plan is complete and syntax is approved:

1. Update the language topics, grammar, and CLR mapping together; add focused
   interface syntax/symbol files rather than enlarging record-only files.
2. Declare source interfaces before binding signatures and bases; resolve cycles,
   accessibility, generics, and inherited member identities.
3. Reuse/refactor source and imported requirement checking, overload resolution,
   nullable safety, and record implementation mapping.
4. Emit interface metadata and documentation, then support source interface calls
   and cross-assembly consumption through the ordinary reference reader.
5. Verify ordinary/generic interfaces, methods/properties, inherited diamonds,
   overloads, constraints, ref/out/in modifiers, nullable metadata, invalid
   implementations, and deterministic output. Build FPAS and C# implementations
   and consumers in both directions through mixed-project SDK tests.

## Scope boundary

Source event declarations, expression-tree conversion, user-authored class
hierarchies, custom awaiters, async iterators, unsafe/byref-like features,
operator declarations, automatic extension-method discovery, and interface
variance/default bodies remain outside the agreed baseline and are not tasks
created by this document. Neither are publishing/AOT/signing, native apphosts,
additional platform/framework coverage, LSP, queries, VS Code, or Visual Studio
project-system integration. Completing an unsupported item mentioned in an old
milestone is not automatically part of completing the original plan.
