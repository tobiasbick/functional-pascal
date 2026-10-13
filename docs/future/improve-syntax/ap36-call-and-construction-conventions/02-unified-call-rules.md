# AP36.2: Unified call rules

Package: [AP36](README.md). Status: complete. AP36.1 records the decisions.

## Result

The [routine-call contract](README.md#recorded-routine-call-decisions) and
[construction/pattern contract](README.md#recorded-construction-and-pattern-decisions)
are implemented. Fixed signatures accept complete positional or named calls;
labels use the visible static receiving signature and compatibility remains
positional. Fixed-arity polymorphic operations publish labels, while variadic
calls remain positional. Enum/Option/Result patterns accept complete positional
or named lists, independently per nesting level. Some/Ok/Error use `Value`.
Every pattern field is explicit. Record defaults, fieldless spellings, AP34
inference, AP35 nonbinding is, and case binding/coverage rules are preserved.

## Implementation owners

- `crates/fpas-parser/src/ast/patterns.rs` and `parser/patterns.rs` retain
  labels and spans in every recursive field list and reject mixed forms.
- `crates/fpas-sema/src/check/calls/named.rs` resolves visible callable labels,
  rejects names on variadic calls, and checks canonical payload arguments with
  contextual types and callable discard/task proofs. Written callable types
  validate case-insensitive unique names in `check/name_resolution/types/resolve.rs`.
- `crates/fpas-sema/src/std_registry/builtins/parameters.rs` supplies the fixed
  polymorphic labels. Existing operation-specific checking retains concrete
  types; unspecialized polymorphic intrinsics must be called directly.
- `crates/fpas-sema/src/check/stmt/control_flow/if_case/patterns/fields.rs`
  checks written field order and normalizes coverage fields to declared slots.
  Existing comparison-before-binding, exhaustiveness and reachability checks
  are reused. Contract, capture and task walkers traverse every payload field.
- `crates/fpas-compiler/src/lowering/case/patterns.rs` extracts named fields
  from declared slots. Existing argument maps preserve receiver-first and
  written-order evaluation, including var-reference indices. Aggregate
  contextual lowering and intrinsic task lowering retain named payload types.
  No VM opcode or unit-interface format change is needed.
- Formatter pattern emission and comment traversal preserve names and nested
  lists; control-body comments retain stable indentation. Project source maps
  remap field-label spans along with their payloads.
- `crates/fpas-language-service/src/intellisense/callable_values.rs` exposes
  concrete static receiving signatures. Signature help and completion handle
  callable aliases, fields, indexed values and pattern labels without proposing
  a named list after positional arguments.
- `crates/fpas-language-service/src/navigation/callable_labels/` resolves
  declared label owners through annotations, aliases, inferred bindings,
  returned signatures, aggregates and nested case payloads. Labels remain
  independent of implementation and binding names. Cycles are bounded and
  imported private fields remain hidden, including indexed record receivers.
- Debugger payload validation accepts canonical named Some/Ok/Error arguments.
  Its documented expression subset still calls function values positionally;
  compiled callable values follow the full static-signature rule.

## Documentation and preserved contracts

The handbook updates [parameters](../../../pascal/language/functions/parameters.md),
[function types](../../../pascal/language/functions/function-types.md),
[patterns](../../../pascal/language/pattern-matching/syntax.md),
[enum construction](../../../pascal/language/types/enums.md),
[Option/Result](../../../pascal/language/types/result-option-types.md),
Math/task APIs, diagnostics, and debugger limits. The grammar and compact
agent reference match implemented behavior. AP09/AP10/AP20 and related older
records include the delivered replacements. AP12 returned-callable chaining
remains deferred; scalar conversions, record patterns, and AP39 coverage
decisions are outside this package.

## Regression coverage

- Compiler named-call and var-parameter tests compare static versus implementation
  labels, callbacks, inferred branch labels, fields/bound methods, arrays,
  eligible go calls, fixed polymorphic operations, and written-order effects.
  Reordered same-typed enum fields and named nested payloads verify actual values.
- Semantic tests cover unique signatures, unknown/duplicate/missing labels,
  per-list mixing, variadic rejection, contextual payload inference, capture
  proofs, coverage normalization, and nonbinding named is in contracts.
- CLI project tests verify imported callable aliases and generic enum patterns
  against fresh and reused compiled-unit sidecars. Existing interfaces retain
  receiving labels without selecting runtime implementation labels.
- Parser and formatter tests cover nested mixed-style rejection, retained spans,
  canonical named fields, comments and repeated-format stability.
- Editor tests cover help/completion/definitions/references/rename for direct,
  annotated/inferred/returned callable signatures, array and record field
  receivers, Option/Result/enum payload roles, nested case selectors, invalid
  cycles, and imported private fields. Field and binding renames stay independent.
- FPAS pattern-resolution fixtures exercise comparison resolution before
  case bindings with reordered named fields. Debugger tests check canonical
  payload labels and rejection of other names.

## Verification

Rust formatting, workspace build/tests, the FPAS suite, source formatting,
example/app project checks, compact-reference examples and documentation links
were checked. Independent review findings were corrected and protected by
regressions. The workspace's existing socket-timeout rounding test remains
platform-sensitive; its failure also occurs on the unchanged baseline.

Both work packages and the central plan are complete. No AP36 decision or
implementation follow-up remains open.
