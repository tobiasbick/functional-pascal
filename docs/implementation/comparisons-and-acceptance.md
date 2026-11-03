# CLR comparisons and specification acceptance

## Comparisons

The six comparison operators (`=`, `<>`, `<`, `>`, `<=`, `>=`) bind applicable
public CLR operators on supported reference and value types, including DateTime,
TimeSpan, Guid, and application types from C# references. Both operand types
contribute candidates. Ordinary conversion and better-overload rules select a
unique operator; ambiguity is an error. Operands execute once, left to right,
and the emitted code calls the selected method.

Built-in numeric, Boolean, string, array identity, and source scalar/aggregate
rules retain their specified behavior. Imported operators cannot be called from
contract conditions. Nullable-value operators are not lifted; validate and unwrap
first. Static-interface/generic-math operators remain in
[language gaps](../future/language-gaps.md).

ComparisonOperatorTests inspects operator MemberRefs and contract/nullable
rejections. ComparisonExecutionTests executes DateTime/TimeSpan/Guid comparisons.
MsBuildTests.Comparisons exercises mixed signatures, both owners, reference/null
equality, evaluation order, and ambiguous imports.
Foreign overloaded equality does not establish nullable-flow facts; runtime
type tests can validate those references. Compiler-defined record/union equality
retains its known null behavior. Regression tests cover branch and short-circuit
guards as well as generated record guards.

## Unsupported metadata

An attempted CLR member with an excluded signature reports FP3005 with its
identity, source location, and unsupported feature: ref returns, byref-like types,
pointers, function pointers, init-only setters, or unsupported required modifiers
and calling conventions. Calls, method groups, delegates, fields, properties,
and indexers are covered. Supported overloads remain usable.
MsBuildTests.SignatureLimits verifies this against a real C# reference.
No signature is erased into object or invoked through reflection/dynamic binding.

## Language acceptance

LanguageSpecificationTests extracts every complete program/unit code fence under
docs/pascal. It compiles the exact sources through the CLI with portable PDB and
XML output, executes programs, and checks output. The two-file example supplies
its documented Geometry unit. New complete examples require an execution assertion.

The file example exercises success and missing-file cleanup; domains/contracts
exercise their specified failures. The unchanged HTTP example uses a local
rejecting proxy for async HTTP exception handling without external network
access. Timer output is checked without requiring a tick count.

| Specification requirement | Test coverage |
| --- | --- |
| CLR identities, visibility, generics, nullable metadata, direct calls | Emission/metadata tests and mixed C#/FPAS builds |
| Arithmetic, aliasing, UTF-16, equality, enums, null flow, explicit annotations | Numeric/Scalar/Collections/Interop/Diagnostics tests |
| Closures, async success/fault/cancellation, disposal and all exits | Callable/NestedRoutine/Async/Interop execution tests |
| Rejected metadata and source diagnostics | SignatureLimits, InteropDiagnostics, and CLI tests |
| Portable symbols, scopes, closures, async stepping metadata | PortableDebugInfoTests and DebugSymbolsCliTests |
| Contracts in Debug/Release, cleanup, delegates, interfaces, C# callers | Contract tests and mixed contracts/async builds |
| Domains, valid defaults, records, globals, cross-assembly contracts | Scalar/Record/Global tests and corresponding mixed builds |
| Comments, XML identities, escaping, warnings, SDK sidecars | Documentation tests and mixed documentation builds |

Debugger interaction is described in the
[portable-symbol implementation](overloads-interfaces-debugging-and-async.md#portable-symbols).
This suite does not declare deferred language-gap items complete.

## Build acceptance

The mixed-project tests verify the initial net10.0/AnyCPU contract: locally
packaged SDK resolution, direct CIL output, C#/FPAS project edges, framework/
assembly/NuGet references, transitive runtime assets, Debug/Release, PDB/XML
copying, incremental inputs/outputs, diagnostics, Clean, and Rebuild.

MsBuildTests.Acceptance additionally checks parallel configurations and independent
FPAS projects, GetTargetFrameworks/GetTargetPath, compiler asset invalidation,
added/removed assembly references, and assembly/PDB/XML equality after clean
rebuilding. Other MsBuildTests files exercise source/reference changes, missing
and corrupted outputs, documentation options, warning policy, and consumers.

Other frameworks/platforms, publishing, AOT, signing, native apphosts, and IDE
integration remain outside this verified initial build scope.
