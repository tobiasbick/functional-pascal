# Binding initialization and local inference

This delivery implements the binding and initializer-inference part of
[stage 4](../stages/04-functional-core.md). It builds on the
[explicit caller-mutation foundation](bindings-effects.md). Later coordinated deliveries are recorded in [checked numbers](checked-numbers.md),
[purity/defaults](purity-and-defaults.md) and [member removal](member-functions.md).
The owning stage records final acceptance.

## Source and semantic ownership

`ast/bindings.rs` owns one initialized `BindingDef` shared by `const` and `var`.
`parser/decl/data/bindings.rs` requires annotations at unit/program scope and
allows them to be omitted in local statements. The obsolete `mutable var`
spelling has a migration diagnostic; `Mutable` itself is an ordinary identifier.
The old duplicate AST declaration/statement variants and lexer keyword are removed.

`sema/check/decl/bindings/` checks initializers, captures inferred enclosing
locals for named nested routines and resolves static scalar values by declaration
identity. Inference uses only the initializer and an optional explicit expected
type. Empty collections, payloadless generic values, unspecialized generic
callables and conflicting decision branches request type context. Named routine
signatures remain explicit. Bindings preserve the initializer's task-bound
callable metadata.

Computed const bindings initialize once at their declaration. Static const
classification is separate from immutability: routine calls and var reads are
non-static, while static references follow lexical shadowing and scope exit.
Compiled interfaces preserve scalar static values. Static aggregates retain their
constant category and load through readonly global storage; computed const
exports use the existing immutable-variable interface category. Object visibility
and imported-global lowering support both forms.

The compiler reuses existing snapshot values, copy-on-write aggregates, closure
cells and resource handles. Local inferred bindings obtain the initializer's
checked type; globals remain explicitly typed. Formatter, source maps and editor
symbols share the binding AST. `language-service/symbols/inference.rs` enriches
editor details, record fields and callable parameter modes from the checked
binding-type map.

## Migration and coverage

An AST-guided conversion changes old readonly `var` bindings to `const` and old
`mutable var` bindings to `var`, preserving formal and actual var markers. It
covers standard source units, applications, examples, tests, Rust source fixtures,
current handbook code and editor sources. Golden formatting, diagnostic ranges,
debugger search markers and editor symbol categories follow the migrated sources.
Current binding pages, grammar, keywords, snippets and FPAS authoring guidance
describe the implemented forms.

Parser and semantic regressions cover optional local annotations, required global
annotations, obsolete spelling, ordinary `Mutable` identifiers, initialization
order, computed/static distinction, inference ambiguity, const write rejection,
procedure values/calls, shadowing, duplicate names and inferred named captures.
Compiler/VM tests cover nested array/dictionary/record copies, value parameters,
decisions and stateful closures. `value_copies_test.fpas` exercises the real runner,
selected native array updates, nested snapshots and shared closure/channel
identity. CLI tests exercise immutable/static/computed imports, located negative
diagnostics, cold/warm unit builds and repeated persisted-program execution.
Editor tests cover inferred record completion and var-callable signatures.

## Verification and remaining work

`cargo fmt --all`, `cargo build` and `cargo test --workspace` pass: 3,778 Rust tests
across 192 result groups, with zero failures. FPAS formatting passes for examples,
tests, applications and source units. The complete FPAS bundle reports 469 passed,
one intentional skip and zero failures. All 109 example/application programs pass
in their appropriate contexts: 95 loose sources and 14 entries covered by 22
passing project manifests. All 57 complete handbook examples pass, including
three examples supplied with their documented companion units.

The real VS Code host passes diagnostics, formatting, navigation, inferred
IntelliSense, semantic tools, project workflows, debugger and lifecycle checks.
TypeScript compilation, grammar/contracts, changed documentation links and the
Git diff check pass. Migrated fixture markers, symbol kinds, diagnostic ranges
and exported aggregate storage are corrected and covered by the final runs.
No stage checkbox is closed by this partial delivery.

The subsequent [checked-number delivery](checked-numbers.md) implements numeric
behavior and records the authorized compiler-identity and static-aggregate
corrections. Coordinated purity/default checking follows numeric verification.
The approved bare-task migration remains coordinated with stage-5 procedure-task
support. Replace member mechanisms after their ordinary-routine replacements work.
