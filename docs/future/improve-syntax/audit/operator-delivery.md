# Operator precedence and bit API delivery

Scope: the operator work item in [stage 3](../stages/03-syntax-and-names.md).
This is a language and standard-library change. The agreed delivery boundary
provides side-effect-free `Std.Bits` functions now; checked purity metadata and
use from pure callables remain with the common [stage-5 checker](../stages/05-effects-and-tasks.md).

## Implementation and ownership

- `crates/fpas-parser/src/parser/expr/precedence.rs` implements comparison above
  `not`, then same-operator logical chains. It rejects mixed chains and obsolete
  infix shifts while preserving recovery and the existing nesting budget.
- Lexer token/keyword tables and parser AST remove `shl`/`shr` operators. The words
  are ordinary identifiers. Comparison-chain guidance explicitly preserves a
  side-effecting middle expression through a single local evaluation.
- `crates/fpas-sema/src/check/expr/operators.rs` accepts only boolean logical
  operands, including statically skipped operands.
- The compiler previously lowered every logical operation eagerly. New
  `crates/fpas-compiler/src/lowering/expr/boolean.rs` owns branch-based `and`/`or`;
  `xor` retains eager left-to-right evaluation. The parent expression dispatcher
  loses obsolete integer operator lowering and its boolean/bitwise selector.
- `crates/fpas-fmt/src/emit/expr/binary.rs` and `mod.rs` mirror precedence,
  left associativity and required grouping. CLI and LSP share that formatter.
- `crates/fpas-sema/src/std_registry/loaded/bits.rs`,
  `crates/fpas-std/src/std_units/symbols/std_symbols/bits.rs`, and the unit registry
  supply the six explicit-import integer signatures. The compiler intrinsic
  catalog maps these names to `crates/fpas-bytecode/src/intrinsic/bits.rs`.
  Existing generic intrinsic selection, verification, and register argument
  windows are reused; no separate selection path is needed.
- `crates/fpas-std/src/bits/` implements bit patterns, count validation, discarded
  left-shift bits and zero-filling right shifts. Ordinary arithmetic is unchanged.
- Debugger expression translation drops old source shift operators; VM watch
  evaluation follows boolean-only short-circuit rules. Intrinsic debug effects
  and recording classify bit calls as side-effect-free.

## Consumer migration

The pre-change parser and semantic metadata were used to inspect repository FPAS
sources and complete Rust-embedded programs. Existing application/library logical
expressions already carry the necessary grouping; no regrouping of those sources
was required. Reviewed right-hand calls are value queries, comparisons, geometry
operations, or cancellation/time observations; no right-hand mutation requires
hoisting to preserve application effects. Short-circuit skipping is the intended
new evaluation rule.

The compiler's existing integer-operator execution fixture now imports `Std.Bits`
with a collision-free alias. Semantic shift tests use bit calls; integer logical
acceptance tests become rejection tests. Old parser/debugger shift acceptance
cases are replaced by migration and bit-call cases. No legacy parse mode or
conversion tool ships with this change.

The grammar, operator/keyword/debugger handbook pages, formatter style, numeric
unit index, `Std.Bits` reference, and FPAS authoring guidance describe implemented
behavior. `lib/api/Std/Bits.fpas` is generated from the registry and handbook.
Existing source templates contain no removed operators and need no rewrite.
VS Code highlighting treats the retired shift words as identifiers.

## Regression coverage

| Area | Added or updated evidence |
|------|---------------------------|
| Parser | `crates/fpas-parser/tests/operator_precedence.rs`: precedence, all mixed logical pairs, parentheses, left associativity, incomplete expressions, bounded nesting, comparison/shift recovery and migration hints |
| Lexer | `crates/fpas-lexer/src/tests/keywords.rs`: retired words, case variants, prefixes, strings and comments |
| Sema | `crates/fpas-sema/src/tests/expr/bits.rs` and `expr/mod.rs`: integer-only signatures, wrong type/arity, alias-only imports, non-static calls, boolean-only logical operands |
| Compiler/VM | `crates/fpas-compiler/src/tests/control_flow/boolean.rs`: left-to-right effects, skipped failures, eager xor, constant initializers, nested groups, `try` continuation, bit argument order, invalid counts and IR validation |
| Bit runtime | `crates/fpas-std/src/bits/tests.rs`: sign/all-bit patterns, every count 0..63, zero shifts, truncation, zero fill, invalid extreme counts, wrong arity/type |
| Wire format | `crates/fpas-bytecode/src/intrinsic/tests.rs`: complete selector list, round trips and global uniqueness |
| Formatter | `crates/fpas-fmt/tests/operator_precedence.rs`: normalized AST, comments and second-format identity |
| Debugger | `crates/fpas-vm/src/vm/debug/evaluation/execute/boolean_tests.rs`: short-circuit call counts, eager xor and integer rejection |
| Real processes | `crates/fpas-cli/tests/json_streams/operators.rs`: parser/sema/runtime JSON errors and linked native runner success/failure |
| Editor | LSP formatting parity and ambiguous-input rejection; generated Bits hover; VS Code grammar assertions |
| FPAS suite | `tests/stdlib/bits/bit_patterns_test.fpas`: public API through the real runner, integer limits and skipped failures |

## Verification

- Targeted lexer/parser/sema/formatter/compiler/std/VM tests passed.
- Real CLI JSON and native-runner tests passed.
- `fpas test tests/ --jobs 8`: 459 passed, 1 intentionally skipped, 0 failed.
- VS Code grammar verification passed.
- `cargo test --workspace --quiet`: 3,420 passed, 0 failed, 0 ignored,
  across 187 test/doc-test suites (including empty suites).
- `cargo build --quiet` and `cargo fmt --all --check` passed.
- `fpas fmt --check tests/`, `examples/`, and `apps/` passed.
- `git diff --check` and relative links in the changed handbook/delivery pages passed.

The operator work item is complete. The
[full coverage delivery](coverage-delivery.md) verifies the final stage-3 gate.
Checked purity metadata retains its stage-5 owner.
