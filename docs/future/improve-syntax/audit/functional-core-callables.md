# Functional-core callable delivery

Scope: stage 4's first implementation checkbox, following the
[reuse audit](functional-core-reuse.md) and
[shared verification](../verification.md).

## Delivered behavior

Existing concrete function and procedure values can be invoked after any typed
expression: returned values, array/dictionary indexes, parenthesized values,
anonymous routines and record fields. Only explicit positional arguments are
passed. The target and its indexes evaluate once before arguments, which evaluate
once from left to right. Saved operands survive control-flow changes caused by
`try`; an error prevents later arguments and the callable body from running.

A procedure value can be stored, passed and returned. A procedure call produces
no value. Function calls used as statements, including final postfix calls, need
a consumer or `discard`. Discard evaluates an ordinary value once and rejects
both a task handle and a type containing task handles in aggregate storage.

Named and anonymous captures retain lexical declaration identity. Immutable
captures are value snapshots; mutable captures share one cell per activation.
Copies of a stateful callable share its environment. Captured mutable parameters
now receive cell-backed local storage at routine entry, preserving caller value
semantics and separate activations. This fixes the F9001 owner-binding failure
reproduced on the pre-delivery compiler.

The existing task invocation path accepts the new final-call forms and preserves
static and runtime task-bound checks. Existing task ownership and detached-spawn
semantics remain owned by stage 5. Selection is exercised through existing
factory statement branches; if/case expression selection belongs to the next
stage-4 slice.

## Ownership and structure

| Concern | Delivered owner |
|---|---|
| Call suffix and discard AST | `fpas-parser`: `ast/expr.rs`, `ast/stmt.rs`, `parser/expr/postfix.rs`, `parser/stmt/basic.rs`; discard keyword in `fpas-lexer` |
| Callable descriptors | Extracted `fpas-sema/src/types/callables.rs` with documented re-exports |
| Argument checks, inference and call metadata | Split `fpas-sema/src/check/calls/{arguments,inference,targets}.rs`; `ValueCallTarget` carries the explicit signature, result and source span |
| Ordinary value invocation | Extracted `fpas-sema/src/check/expr/calls/values.rs`; named, member and postfix paths reuse it |
| Member dispatch | Split affected oversized dispatcher into `check/expr/calls/methods/{resolution,arguments}.rs` |
| Captures | Extracted `check/closures/capture/traversal.rs`; traversal includes target, index, argument and discard operands |
| Result consumption | New `check/stmt/discard.rs`; statement calls and postfix chains share result-use checking |
| Lowering | New `fpas-compiler/src/lowering/calls/values.rs`; existing `CallValue`, saved values, IR/bytecode verification and VM invocation are reused |
| Parameter capture owners | Extracted `lowering/context/initialization.rs`; entry creates cells for captured parameters |
| Tooling | Formatter emission/comment traversal, project source-ID mapping, debug expression validation and navigation handle the new AST forms; `fpas-language-service/src/intellisense/signature_help/values.rs` resolves computed signatures |

No new bytecode instruction, parallel invocation implementation or compatibility
mode is introduced. Member mechanisms remain available until their separately
ordered replacement slice.

## Coverage

| Cases | Evidence |
|---|---|
| Returned/indexed/parenthesized/anonymous/literal/field targets and stored procedures | `crates/fpas-compiler/src/tests/calls/targets.rs`; `tests/stdlib/closures/callable_targets_test.fpas` |
| Target/index/argument/body effect order; exactly-once discard; `try` forwarding | `calls/targets.rs`: `TABC`, `TIABC` and early-error `TA` traces |
| Shared copied environments, named/anonymous parity, parameter cells, independent activations, caller snapshot, lexical shadowing, loop values and nested collection snapshots | `calls/captures.rs` plus the FPAS runner fixture |
| New task targets and dynamic mutable-capture rejection | `calls/tasks.rs`; existing VM transfer/owner regressions remain in the workspace suite |
| Wrong arity/types, noncallable targets, invalid procedure value positions, unused ordinary/intrinsic/postfix results, nested task discard and static task-bound propagation | `crates/fpas-sema/src/tests/expr/callable_targets.rs` |
| Positional-only arguments, malformed suffixes and missing operands | `crates/fpas-parser/src/tests/expr/postfix.rs` |
| Imported factories, callable fields without a receiver, imported mutable parameter captures, private callable fields, located JSON diagnostics | `crates/fpas-cli/src/main_tests/projects/callable_targets.rs` |
| Comment preservation, multiline calls, wrapping and idempotence | `crates/fpas-fmt/tests/callable_targets.rs` and existing corpus round trips |
| Computed signature help and discard highlighting | `crates/fpas-language-service/tests/intellisense.rs`; VS Code grammar fixture and `verify:grammar` |

## Consumers and current documentation

Consumer inventory covers `lib/`, `apps/`, `examples/`, `tests/`, Rust-embedded
sources, handbook examples, editor integration, source templates and repository
authoring guidance. Migration uses semantic diagnostics and resolved project
imports. Ordinary ignored values receive `discard`; task-returning start helpers
receive handle bindings. Group-lifecycle coverage explicitly lets those bindings
leave scope and still observes the grandchild's completion at group close.
Runtime-negative fixtures consume ordinary values so they continue reaching their
original runtime checks. Expected diagnostics and runtime assertions are retained.
Temporary repository conversion tooling is removed.

All 132 app/example source and project checks pass. Single-file sources that need
dependencies are checked through their owning manifest. Source-library APIs and
intrinsic declarations retain their signatures; no regeneration is needed.

Current pages updated: first-class callables, callable types, closure captures,
postfix chaining, routine declarations, keywords, task examples, formatter style
and `docs/specs/grammar.ebnf`. New constructs in those pages are implemented.
Generic data, decision expressions, bindings/inference, caller references, purity
and member removal remain in their owning stage checkboxes.

## Verification

| Check | Result |
|---|---|
| `cargo fmt`, `cargo build` | Passed |
| Targeted parser/sema/compiler/formatter, CLI and signature-help coverage | Passed; ten new compiler execution tests and three new CLI tests |
| `cargo test --workspace` | Passed, including 532 CLI tests, 580 VM unit tests and nine process-lifecycle tests |
| `fpas fmt --check lib apps examples tests` | Passed |
| `fpas test --std-lib lib tests/suite.fpasprj --report json --jobs 4` | 461 passed, one intentional runner skip, zero errors/failures |
| App/example CLI checks | 132 passed |
| `npm run verify:grammar` in `editors/vscode` | Passed |
| Documentation links and `git diff --check` | Passed |
