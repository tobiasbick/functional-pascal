# Stage-3 coverage delivery

Scope: the final work item in [syntax and names](../stages/03-syntax-and-names.md).
This delivery checks the implemented grammar, names, evaluation, diagnostics,
and tooling against that stage's requirements. It adds regression coverage,
corrects diagnostic examples, and tightens validation of the existing name/value
rules. The specified language syntax, semantics, and APIs are unchanged.

## Block-table coverage

The parser's `tests/syntax_and_names/boundaries.rs` contains a table of 16
named/plain block cases. It includes separate record and enum declarations,
both `for` forms, and record literals alongside the stage's record updates.
Every case accepts its canonical closer, rejects its removal and every other
implemented closer, rejects a missing final body/declaration terminator, and
rejects an extra separator before its closer. Statement/declaration closers also
require their own final semicolon. Anonymous routines and record expressions
close directly before their caller's comma or parenthesis.

Repeat has separate tests for `until`, missing/wrong/redundant closers, the body
and condition terminators, a nested conditional, and the following statement.
Every rejected closer has bounded recovery; named/plain closer failures must
identify the correct owner. Additional empty-body cases cover named/anonymous
callables, collection loops, `elsif`, `else`, and case fallbacks. Empty units and
record declarations/literals remain valid; empty enums and updates are rejected.
Grouped binding names and formal parameters produce one diagnostic with an
example that names each declaration/parameter individually.

Paths below are relative to the named crate unless a repository path is given.
The common parser matrix above supplies negative boundary coverage for each row.

| Implemented stage-3 row | Additional positive, structural, or execution evidence |
|---|---|
| Program | Parser `tests/syntax_and_names/blocks.rs`; formatter `tests/syntax_and_names.rs`; compiler `tests/syntax_and_names.rs`; actual CLI handbook execution |
| Unit | Parser matrix; formatter named-unit and empty-unit comment regressions; CLI `src/main_tests/fmt/source_conversion.rs` resolves, formats, builds, and executes two source units |
| Function | Formatter nested functions; compiler `src/tests/functions.rs` direct calls, recursion, early returns, nested routine calls, and callable values |
| Procedure | Formatter nested procedures; compiler `src/tests/functions.rs` procedure calls and procedure callbacks |
| Anonymous function | Parser argument boundaries; formatter callback round trips; compiler `tests/syntax_and_names.rs` closures inside updates; actual handbook callback execution before/after formatting |
| Anonymous procedure | Parser argument boundaries; formatter procedure arguments with comments/repeat; compiler `src/tests/closures.rs`; actual handbook callback execution |
| Plain lexical block | Parser plain versus named ownership; formatter explicit blocks; semantic escaping-name negatives; compiler `all_loop_and_case_bodies_execute_multiple_statements`; CLI before/after scope checks |
| Record / enum declaration | Separate parser cases; formatter declarations; semantic whole-unit types and imported variants; compiler record construction/update; CLI source units with identically named enum exports |
| Conditional statement | Parser branch lists, multiple `elsif`, nested `else if`, and missing-nested-closer guidance; semantic branch isolation; compiler condition count/order, branch returns, and nested conditionals |
| Case statement | Parser arm/fallback boundaries; semantic distinct arm locals, pattern/guard bindings, and escape rejection; compiler scalar/variant arms; actual handbook case execution |
| Counted loop | Parser ascending/descending loops; formatter descending nested case/if; semantic iterator scope and alias collisions; compiler multiple statements and nested break/continue |
| Collection loop | Parser collection bodies and empty-body rejection; semantic iterator scope/collisions; compiler arrays/dictionaries and multiple statements |
| While loop | Parser named boundary matrix; formatter round trips; explicit plain-block visibility inside a loop; compiler loop bodies and nested break/continue |
| Repeat loop | Separate parser boundary tests; formatter callback/nested bodies; semantic condition lookup outside the body; compiler `src/tests/control_flow/repetition.rs` outer bindings after fallthrough/continue and different-type body shadows |
| Record update | Parser update/literal nesting and malformed initializers; formatter expression/argument boundaries; semantic private-member rejection; compiler copy-preserving update; CLI imported nested literals before/after formatting |

Conditional and case expressions remain assigned to stage 4; task scopes remain
assigned to stage 5. Their grammar, migration, and tests land with those stages,
as already specified in stage 3. Existing method/property/event syntax and
binding rules retain their later-stage migration boundaries.

## Names and visibility

`crates/fpas-sema/tests/syntax_and_names.rs` checks:

- Alias-only access for intrinsic/source units, case-insensitive dispatch,
  unchanged lexical routine names after unrelated imports, and rejected
  short/full/nested-unit access paths.
- Duplicate units/aliases, every declaration/parameter/iterator collision kind,
  nested named routines, anonymous parameters, and Option/scalar-guard pattern
  bindings that collide with an import alias. Conflicting variable declarations
  produce the alias diagnostic without unrelated follow-on errors.
- Import replacement hints preserve the source spelling of aliases and member
  names, including short-name and full-unit-path mistakes.
- Distinct if/case scopes, pattern names inside guards and arm bodies, and names
  escaping into neighboring guards, fallbacks, or subsequent statements.
- Whole-unit forward types in signatures, fields, aliases, and recursive
  collections, with rejected unknown types/cycles and declaration-order value
  and field-default initialization.
- Private source-unit types and record members. Public factories may initialize
  private fields and call private routines within their owner. Imported clients
  can read public fields, but private reads, assignment, updates, and direct
  construction fail with `F2017`, including case variants.
- Counted/collection iterator visibility, explicit block scope inside `while`,
  closure locals, and repeat-body names unavailable in the `until` condition.
- Type names rejected as value expressions and as record-update bases. Type
  qualifiers for valid static calls and variant constructors remain available.

Existing CLI `src/main_tests/visibility/` tests additionally execute internal
private calls and reject private exports through both bare and qualified paths.
CLI source-conversion tests build and run two units with identical exported
names while preserving their explicit resolved owners and a same-named local.

## Precedence, effects, and bit boundaries

`crates/fpas-parser/tests/operator_precedence.rs` now covers each multiplicative
operator above both additive operators, multiplicative left associativity, all
seven comparisons below arithmetic and above `not`, and all 49 ordered pairs
of rejected chained comparisons. Postfix call/index/field chains and record
updates bind above unary minus/`try` and arithmetic. Existing tests cover
same-operator logical chains, every mixed pair with both valid parenthesizations,
bounded nesting, obsolete shift guidance, and recovery of following statements.

Formatter operator tests compare normalized ASTs and second-format identity,
including prefix/postfix/update boundaries, non-associative arithmetic grouping,
and explicitly parenthesized comparisons. Block/comment/corpus round trips
likewise preserve structure, comments, and idempotence.

Compiler `src/tests/control_flow/boolean.rs` executes chains that stop at the
middle operand and chains that reach the last operand. Trace assertions check
the result, written order, and exactly-once evaluation. Existing tests execute
eager xor, nested grouping, skipped and active failures, constant initializers,
`try` continuations, bit-call argument order, and IR validation before/after
optimization. VM debugger tests check short-circuit call counts and boolean-only
operands; bytecode tests check selector completeness, round trips, and uniqueness.

`tests/stdlib/bits/shift_counts_test.fpas` exercises every count 0 through 63
through the public API and actual test runner. Its expectations use arithmetic
recurrences rather than another shift implementation. It checks positive/sign
patterns, zero operands, both integer extrema at count zero, and sign-bit
combination with BitAnd/BitOr/BitXor. Existing bit-pattern tests cover BitNot,
discarded left-shift bits, zero fill, and skipped failures.

Real CLI JSON tests execute both shifts with counts at the integer minimum,
`-1`, `64`, `65`, and the integer maximum, including zero operands. They require
runtime phase, `F4012`, and failure exit code 2. Existing linked native-runner
tests execute short-circuit success and invalid-count failure; semantic/std
tests cover wrong types/arity, alias-only lookup, and statically skipped invalid
logical operands.

## Tooling and documentation

Existing LSP tests run the shared formatter through unsaved buffers, preserve
comments, verify second-request identity, and refuse malformed input. Snippet
tests cover every current source template and negative/edge forms. The actual
VS Code extension-host suite checks formatting, diagnostics, highlighting,
navigation, completion, projects, and debugger integration.

The empty-record-update and empty-enum parser hints now teach `end with` and
`end enum`. Negative tests require these named closers, and positive replacement
examples must parse. The routine declaration handbook clarifies individually
typed formal parameters alongside the grouped-name diagnostic. Other handbook
and grammar changes belong to the coordinated documentation delivery.

## Verification

- This delivery adds 19 Rust regression tests and one FPAS test program, and
  extends the existing formatter expression matrix and diagnostic regression.
- Targeted parser, semantic, formatter, compiler, and actual CLI tests pass.
- `cargo fmt`, `cargo fmt --all --check`, and `cargo build --quiet` pass.
- `cargo test --workspace --quiet`: 3,461 passed, zero failed, zero ignored,
  across 187 test/doc-test suites, including empty suites.
- `fpas fmt --check lib apps examples tests` passes all 823 source files.
- `fpas test tests/suite.fpasprj --jobs 8`: 460 passed, one intentional skip,
  zero not run, zero failed.
- `node editors/vscode/scripts/verify-grammar.mjs` passes.
- `node editors/vscode/scripts/run-tests.mjs` passes the complete actual
  extension-host suite, including diagnostics, formatting, navigation,
  IntelliSense, semantic tools, projects, debugger, and lifecycle checks.
- Relative file links and changed Rust documentation links resolve.
  `git diff --check` passes. No machine-specific metadata is recorded.

## Completion boundary

Only implemented stage-3 constructs are covered by this delivery. Completing
this gate closes the final stage-3 work item and the central stage checkbox.
Stage 3 is complete. Next: the stage-4 callable/type facilities in the
[bounded sequence](implementation-sequence.md), coordinated with stage-5
references/purity before binding/default migration. Later-stage constructs keep
their own acceptance and migration requirements.
