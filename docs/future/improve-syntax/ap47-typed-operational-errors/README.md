# AP47: Native typed operational errors

Status: AP47.1-AP47.5 complete; later packages remain planned.
The user selected native `error` declarations, automatic error
output, explicit MapError and unchanged try, and delegated the remaining design
choices. This plan records those choices. The runtime, language, constructor-value, explicit-mapping
and host/parsing slices are delivered. Task/network migration and final acceptance
require their own implementation authorization.

Effort: large across compiler, runtime, standard library and consumers.
Completion is tracked only in the [central plan](../README.md#package-status).

## Goal and selected contract

Authors declare an error family once using Pascal-shaped syntax. The language
supplies nominal identity, exhaustive matching, ordinary constructor values,
readable output and typed causes. Applications require no reporting adapter,
registration, custom derivation framework or class hierarchy.

```text
type ConfigError = error
  FileFailure(Cause: Fs.FsError) = 'Cannot load configuration';
  MissingSetting(Name: string) = 'Required setting is missing';
end error;

const Text := try Fs.ReadText(Path).MapError(ConfigError.FileFailure);
```

The error declarations, MapError operation and typed Fs interface in this
combined example are implemented. The complete
contract is [AP47.1](01-decision.md); the standard-library contract is
[standard-errors.md](standard-errors.md).

## Current checkout and preserved behavior

- Error remains a Result constructor. AP47.3 adds contextual native error
  declarations, the Error constraint, typed construction/matching and Message().
- Enum layouts and shared enum values already provide nominal variants and
  payload storage. Runtime values have a stack-safe display implementation.
- AP47.2 supplies essential native-error metadata, typed causes, automatic
  rendering and portable constructor factories without exposing source syntax.
- Result supplies Map, MapError, AndThen, OrElse and Unwrap. Payload enum and
  error constructors are ordinary callable values.
- Host/parsing [Std interfaces](../../../pascal/std/README.md), including JSON/TOML
  field helpers, return native errors. Tasks, nested selection callbacks and
  network/HTTP interfaces retain string errors until AP47.6.
- Preserve Result and Option as ordinary data, compatible-type try propagation,
  evaluation order, task ownership, cancellation priorities and runtime failures.
- Preserve application-authored `Result of (T, string)` and other Result error
  payloads. Native error families are an available capability, not a new
  constraint on every Result.

The selected design has no public ErrorContext/ErrorFrame/ErrorReport layer,
handwritten Describe functions or opt-in deriving syntax. Shared native failure
information is an ordinary leaf error, not a reporting framework.

## Decisions and authority

Q28/Q29 record the user's replacement direction. Q30-Q36 are concrete choices
made under the user's delegation to prepare this implementation plan. The later
implementation requests authorize AP47.2, AP47.3, AP47.4 and AP47.5. Later packages still
require their own authorization.

| ID | Selected decision | Contract owner | Affected work packages |
| --- | --- | --- | --- |
| Q28 | Native `error` declarations with nominal variants; no inheritance or erased root Error value | AP47.1, declaration and type rules | AP47.2, AP47.3 |
| Q29 | Automatic output and one typed Cause field; no custom reporting functions | AP47.1, output and causes | AP47.2, AP47.3 |
| Q30 | Error is a built-in generic constraint implying Printable; concrete error types remain in Result | AP47.1, generics | AP47.2, AP47.3 |
| Q31 | Optional literal message per variant; payloads displayed automatically; Message() and printing share one renderer | AP47.1, messages and output | AP47.2, AP47.3 |
| Q32 | Payload constructors are ordinary function values for both enums and errors; MapError explicitly maps any Result error payload; try unchanged | AP47.4 | AP47.3, AP47.4 |
| Q33 | Per-domain error catalogs plus one native HostError leaf; classify from structured native outcomes | standard-errors.md | AP47.5, AP47.6 |
| Q34 | Replace selected Std signatures in place with all affected consumers; no parallel legacy wrappers | AP47.5, AP47.6 | AP47.5, AP47.6 |
| Q35 | Reuse shared enum values with concrete error instance keys and hidden generic constructor factories; preserve ordinary enum erasure and 16-byte Value; version essential metadata and gate regressions | AP47.2, AP47.7 | AP47.2-AP47.7 |
| Q36 | Seven coherent delivery packages, one completion index and implementation authorization bounded to the selected work package | This README | AP47.1-AP47.7 |

## Dependencies and work packages

The design contract, runtime/artifact foundation and source-language slice are
complete, including constructor values, MapError and host/parsing migration.
Task/network migration and final acceptance remain open.

- [x] [AP47.1: Language and error contract](01-decision.md)
- [x] [AP47.2: Runtime layouts and portable artifacts](02-runtime-and-artifacts.md)
- [x] [AP47.3: Language, compiler and editor support](03-language-and-tooling.md)
- [x] [AP47.4: Constructor values and MapError](04-maperror-and-constructors.md)
- [x] [AP47.5: Host, parsing and related Std migration](05-host-and-parsing.md)
- [ ] [AP47.6: Tasks, networking and layered consumers](06-tasks-and-network.md)
- [ ] [AP47.7: Final acceptance, documentation and performance](07-acceptance-and-performance.md)

Order: AP47.1 -> AP47.2 -> AP47.3 -> AP47.4 -> AP47.5 -> AP47.6 -> AP47.7.
AP47.2 adds dormant metadata support without exposing syntax that cannot run.
AP47.3 delivers a usable language slice before Std signatures change. Each Std
cohort migrates its transitive callers in the same package and must leave a
buildable checkout. A package may be subdivided before implementation only if
its resulting slices each preserve this requirement.

Prerequisites are current records/enums, generic constraints, Result/try and
hosted callback execution. No other FPAS 2 proposal, including numeric widths,
cleanup actions, process handles or FFI, is a prerequisite. Numeric examples use
currently implemented integer types; a separate numeric migration changes names
later without changing the error contract.

## Open decisions and exclusions

No language or interface choices are left for an implementer to guess within
this package. Later work packages still require implementation authorization. Profiling, exact
internal file splits and diagnostic-code allocation are implementation work,
not requests to reopen the selected contract.

Exclude automatic error conversions, exception handling, stack capture, user
traits, custom error formatting overrides, localization, logging backends,
branching cause graphs, new I/O/process capabilities and a general decoder
framework. Existing structured runtime/task-failure reports retain their roles.

## Completion criteria

Every package must satisfy its own acceptance criteria. Final delivery includes
all selected Std signatures, callback shapes, artifacts, generated declarations,
source consumers, editor support and executable handbook examples. No selected
condition may require parsing a diagnostic message. Changed message wording
must not change branching. Runtime panics and contracts must stay runtime
failures rather than being caught or relabeled as operational errors.

The [acceptance plan](07-acceptance-and-performance.md) owns verification and
performance gates. Package completion, commit and publication are separate.
Only this README owns AP47's work-package checklist.
