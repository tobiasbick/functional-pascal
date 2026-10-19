# AP47: Native typed operational errors

Status: Planned. The user selected native `error` declarations, automatic error
output, explicit MapError and unchanged try, and delegated the remaining design
choices. This plan records those choices. Implementation is not authorized.

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

This is selected future syntax, not current implemented behavior. The complete
contract is [AP47.1](01-decision.md); the standard-library contract is
[standard-errors.md](standard-errors.md).

## Current checkout and preserved behavior

- Error is currently a Result constructor; `type T = error ... end error` and
  the built-in Error constraint are not implemented.
- Enum layouts and shared enum values already provide nominal variants and
  payload storage. Runtime values have a stack-safe display implementation.
- Result supplies Map, AndThen, OrElse and Unwrap. MapError is not implemented.
- Many [Std interfaces](../../../pascal/std/README.md) return string errors,
  including nested selection callbacks and source HTTP/field helpers.
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
made under the user's delegation to prepare this implementation plan. They do
not claim separate user approval of each detail or permission to implement.

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
| Q36 | Seven coherent delivery packages, one completion index and planning-only authorization | This README | AP47.1-AP47.7 |

## Dependencies and work packages

The only completed item below is the recorded planning contract. All source
implementation, migration and acceptance packages remain unchecked.

- [x] [AP47.1: Language and error contract](01-decision.md)
- [ ] [AP47.2: Runtime layouts and portable artifacts](02-runtime-and-artifacts.md)
- [ ] [AP47.3: Language, compiler and editor support](03-language-and-tooling.md)
- [ ] [AP47.4: Constructor values and MapError](04-maperror-and-constructors.md)
- [ ] [AP47.5: Host, parsing and related Std migration](05-host-and-parsing.md)
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
this package. Implementation authorization is still required. Profiling, exact
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
