# Stage 3 syntax and consumer inventory

This is the pre-change inventory for [stage 3](../stages/03-syntax-and-names.md).
The inspected checkout is `c5b6c1a7`. Paths in code spans are repository-relative.
The observations below describe that baseline, before grammar, source, handbook,
and editor migration. The subsequent implementation is recorded separately in
[block syntax and names delivery](block-syntax-delivery.md).

## Lexer and parser ownership

`crates/fpas-lexer/src/token/kind.rs` defines token kinds;
`crates/fpas-lexer/src/token/keywords.rs` maps complete identifiers using
ASCII-case-insensitive matching. `lexer/identifiers.rs`, `lexer/symbols.rs`,
`lexer/trivia.rs`, and `comments.rs` in that crate own scanning, punctuation,
trivia, and comment spans. Keyword prefixes remain identifiers. `as`, `elsif`,
`when`, and `null` currently tokenize as identifiers; their stage-3 constructs
need keyword handling and matching diagnostics together. `pure`, `scope`, and
`discard` are also currently identifiers and belong to later stages.

`mutable`, `property`, `read`, `write`, `event`, `nil`, `shl`, and `shr` are still
tokens with current-language roles. Keep their removal tied to the owning
implementation/migration, including member replacement in stage 4. Do not scan
raw substrings to diagnose obsolete syntax: identifiers, strings, and comments
must remain distinguishable. `crates/fpas-parser/src/parser/display.rs` is the
exhaustive token-to-diagnostic spelling consumer.

The following paths are below `crates/fpas-parser/src/`. Every current construct
in the stage-3 block table has an owner here; future constructs are identified
separately.

| Construct / rule | Current owner and behavior | Later migration boundary |
|---|---|---|
| Program | `parser/program.rs`, `parse_program_ast`: `begin`, statement list, `end`, dot, then EOF | Require `end program;`; preserve trailing-input rejection and full source span. |
| Unit | `parser/program.rs`, `parse_unit_ast`: declarations followed by EOF, with no closing token | Add `end unit;` without admitting a main body. |
| Named and nested function/procedure | `parser/decl/routines.rs`: both bodies use `parse_func_body`, consuming `end;`; nested declarations use the same parsers | Pass the routine kind to closing-token validation; preserve nested declaration boundaries. |
| Anonymous function/procedure | `parser/expr/closure.rs`: `parse_closure_body` consumes `end` without a following semicolon | Add the routine-specific closer, keeping surrounding comma, parenthesis, or statement terminator outside the expression. |
| Plain lexical block | `parser/stmt/basic.rs`, `parse_block`: `begin`, statement list, `end` | Preserve a distinct lexical block and its own terminator inside a named construct. |
| Record declaration | `parser/decl/data/record_members.rs`: record fields, methods, properties, events, then `end`; `type_defs.rs` consumes the declaration semicolon | Add `end record`; retain member mechanisms until stage 4 replaces them. |
| Enum declaration | `parser/decl/data/type_defs.rs`: semicolon-terminated members and `end`, then the type declaration semicolon | Add `end enum`; keep payload/member recovery separate from outer declaration recovery. |
| If statement | `parser/stmt/branching.rs`: one statement for each branch; `else if` is a nested `Stmt::If`, with no named closer | Add branch lists, `elsif`, and `end if`; preserve actual nested ownership and distinguish explicit blocks. |
| Case statement | `parser/stmt/branching.rs`: labels and optional guard followed by one body statement; optional `else` list; `end` | Add `when`, body lists, and `end case`; distinguish arm boundaries from nested statements. |
| Counted / collection for | `parser/stmt/loops.rs`: typed header, `to`/`downto` or `in`, then one statement | Add body list and `end for`; preserve counted direction, iteration binding, and collection evaluation. |
| While | `parser/stmt/loops.rs`: condition, `do`, one statement | Add body list and `end while`. |
| Repeat | `parser/stmt/loops.rs`: statement list and `until` expression | Keep `until` as the closer; require the last body statement and the loop statement to terminate independently. |
| Record update | `parser/expr/precedence.rs` enters `parser/expr/primary.rs`, `parse_record_update`; field initializers end with semicolons and bare `end` | Add `end with`, retaining nonempty updates, nested expression boundaries, and field order. |
| If/case expressions; task scope | No corresponding variants in `ast/expr.rs` / `ast/stmt.rs` | Stage 4 owns decision expressions; stage 5 owns scope. Do not count these as implemented stage-3 productions. |
| Declaration groups | `parser/decl/mod.rs`, `decl/data/const_var.rs`, `decl/data/type_defs.rs` collect repeated identifiers after a single keyword; visibility is carried through the group | Replace groups with one keyword per declaration, including `public`; preserve declaration order. |
| Formal versus actual arguments | `parser/decl/routines.rs`, `parse_formal_param_list` expects individually typed, semicolon-separated parameters; `parser/expr/mod.rs`, `parse_arg_list`, handles comma-separated arguments | Retain these separators; improve grouped/comma-formal diagnostics without treating actual arguments as declarations. |

`ast/stmt.rs` stores `If`, `For`, `ForIn`, and `While` bodies as boxed individual
statements; `CaseArm` has one `Stmt`. `Repeat` and `Case.else_body` already use
lists. There is no `Null` statement. AST changes must reach semantic checking,
lowering, formatter traversal, and editor symbol/selection traversal together.
`crates/fpas-sema/src/check/stmt/mod.rs` gives `Stmt::Block` its own scope; a
migration must not erase explicit blocks merely because branches gain lists.

## Recovery and expression boundaries

- `crates/fpas-parser/src/parser/stmt/mod.rs`, `parse_statement_list`, currently
  accepts empty lists and optional final semicolons. Its end set is `End`,
  `Else`, `Until`, `Eof`. `recover_statement_separator` also uses that set and
  `can_start_statement`; a separator-only statement is diagnosed. Named closers,
  `elsif`, `when`, and `null` require coordinated end/start and recovery handling.
- `parser/core.rs` in that crate owns `expect`, expected/found details,
  `is_expression_recovery_boundary`, and synthetic EOF insertion. `expect` does
  not consume a mismatched token. Preserve parent delimiters and guarantee
  progress when an inner construct encounters an outer or mismatched closer.
- `parser/program.rs`, `is_qualified_id_recovery_boundary`, protects declaration
  and control-flow tokens while recovering dotted names. Extend this alongside
  alias grammar, not only the successful import path.
- `parser/stmt/branching.rs` has separate case-arm stop/recovery logic. A label
  after a missing separator currently remains available for the next arm.
  Keep that property when arms start with `when`.
- `parser/nesting.rs` limits recursive expression, statement, type, and routine
  parsing. Keep its shared budget and EOF behavior when adding closer recovery.
  Reuse stage-2 diagnostic records and codes; do not add a compatibility parser.

The expression precedence owner is
`crates/fpas-parser/src/parser/expr/precedence.rs`: comparisons are currently
below additive `+`, `-`, `or`, `xor`; multiplicative operators include `and`,
`shl`, `shr`; `not`, minus, and `try` share recursive unary parsing. Chained
comparisons are already rejected and their extra operands consumed for recovery.
`crates/fpas-fmt/src/emit/expr/binary.rs` duplicates precedence for printing and
must change with the parser.

Inspection also confirms that
`crates/fpas-compiler/src/lowering/expr.rs`, `lower_binary`, routes all three
logical operators through `lower_direct_binary`, which lowers both operands.
That is evidence for the later operator work, not a short-circuit implementation.
The new rules must test evaluation count as well as grouping, including
side-effecting middle operands in rejected comparison chains.

## Imports and name consumers

The current import representation is `Vec<QualifiedId>` in
`crates/fpas-parser/src/ast/program.rs`. `parse_uses_and_declarations` accepts
one optional comma-list clause; `QualifiedId` has path parts and a span, with no
alias or alias span. Unit identity and local qualifier must become separate
information without making an alias a different dependency.

| Consumer | Current dependency to migrate |
|---|---|
| `crates/fpas-sema/src/check/entry.rs` | Builds used-unit names from `parts`, registers imported symbols, then checks declarations in order. Whole-unit type collection must be separate from initializer evaluation order. |
| `crates/fpas-sema/src/std_registry/aliases.rs`, `check/name_resolution/std_names.rs` | Registers/resolves short standard-library names and ambiguity candidates, while retaining fully qualified access and lexical shadowing. Replace source access paths with the explicit alias policy. |
| `crates/fpas-sema/src/interface/install.rs` | Installs full exported names and short source-unit candidates, including enum variants. Supporting interface types must remain available internally without becoming a second source-level import path. |
| `crates/fpas-project/src/unit_graph/resolve.rs`, `model.rs`, `program.rs` | Traverses direct uses and validates export/reachability rules using canonical unit names. Keep dependency keys, public export rules, and diagnostic source attribution independent of alias spelling. |
| `crates/fpas-language-service/src/navigation/document.rs`, `resolve.rs` | Stores imported owner strings and resolves short/full names for navigation. Alias declarations/references must reach definitions, rename, completion, semantic tokens, and selection ranges consistently. |
| `crates/fpas-language-service/src/intellisense/auto_import.rs` | Replaces the first `uses` clause through its first semicolon; appends a `QualifiedId`, formats an AST copy, and extracts that clause. Commented clauses are deliberately refused. Replace this algorithm when imports become separate alias declarations. |

Alias collisions, duplicate units under different aliases, case-insensitive
shadowing, and type/routine name collisions need semantic tests, not merely new
lexer/parser cases. Keep current visibility checks. See the existing
[source map](source-map.md) for compiler/interface ownership beyond syntax.

## Formatter and comments

All paths in this section are below `crates/fpas-fmt/src/`.

| Owner | Current behavior / required coordination |
|---|---|
| `emit/program.rs` | Prints `end.` for programs, no unit closer, and a shared comma-list `uses` clause. Commented imports have a separate emission path. |
| `emit/decl/group.rs` | Coalesces adjacent declarations by kind/visibility and emits grouped keyword headers; must stop regrouping migrated declarations. |
| `emit/decl/item.rs` | Prints record/enum endings and shared function/procedure body endings, including record methods. Keep those callable closers consistent during the stage-4 transition. |
| `emit/stmt/mod.rs`, `line.rs`, `loops.rs` | `is_last` suppresses the last semicolon; branches are wrapped in `begin/end`; nested else-if is printed as `else if`. Replace these conventions together with the AST. |
| `emit/expr/closure.rs`, `literal.rs`, `mod.rs`, `binary.rs` | Own expression closers, record-update formatting, precedence, and parentheses. A closure argument must still close immediately before the argument delimiter. |
| `comments/anchors.rs`, `map.rs` | Uses a single `uses_anchor`, finds the first `Uses` token, and permits only whitespace/semicolons between a construct end and a same-line comment. Multiple imports and two-token closers need accurate distinct spans/anchors. |
| `comments/traversal.rs`, `traversal/expressions.rs` | Collects AST spans, `Uses`/`Begin` token anchors, routine headers, and callable-body anchors; new branch/closer boundaries must retain comment ownership. |
| `lib.rs`, `span.rs` | Source-aware formatting validates spans and source/AST identity. AST-only formatting cannot recover comments. Preserve this distinction during conversion. |

`tests/common/mod.rs` in the formatter crate verifies successful parsing before
and after formatting, comment text/order, and second-format identity. It does
**not** compare normalized AST structure before and after formatting. Add
structural and scope-sensitive checks with the later implementation; textual
idempotence alone cannot prove branch ownership or lexical-scope preservation.
The tree tests cover `examples/`, `tests/`, and `apps/`, not `lib/` or editor
fixtures. Do not place malformed fixtures in that valid-source corpus.

## Generated sources, assets, and guidance

| Producer / consumer | Audit result and migration boundary |
|---|---|
| `crates/fpas-cli/src/cli_init/templates.rs` | Program, library, and workspace scaffolds emit current closers, plain imports, unqualified calls, and unterminated final body statements. Update both library and consuming program templates. |
| `crates/fpas-sema/examples/export_intrinsic_std_api.rs` and its `export_intrinsic_std_api/documentation.rs` | Generates grouped public constants/types and callable panic stubs, using registry types and handbook signatures; parses and formats each unit before writing `lib/api/Std/`. Change the producer and inputs, then regenerate the declarations; do not hand-maintain only the output. |
| `crates/fpas-bench/src/native/compiler.rs`, `language_service.rs`, `program_artifact.rs`, `project_queries/fixture.rs` | Builds synthetic source for compiler, language-service, artifact, and multi-unit workloads. These remain grammar consumers even though benchmark execution is outside this audit. |
| `editors/vscode/snippets/fpas.json` | Eleven snippets cover program, unit, function, procedure, record, mutable variable, if, for, while, repeat, and case. Empty placeholders, old closers, and omitted final terminators need migration. |
| `editors/vscode/syntaxes/fpas.tmLanguage.json` | Has declaration-group contexts and explicit keyword/operator lists; current control words omit the new stage-3 words. Update contexts as well as the keyword list. |
| `editors/vscode/language-configuration.json` | Indentation increases for `begin`, `record`, `enum`, `repeat`; decreases for `end`, `until`, `else`. New branch-list headers and `elsif`/`when` need matching editor rules. |
| `crates/fpas-language-service/src/intellisense/completion.rs`, `semantic_tools/tokens.rs` | Keyword completion uses explicit context lists; semantic tokens classify resolved identifiers. Adding lexer keywords alone does not migrate completion or alias classification. |
| `crates/fpas-language-service/src/formatting.rs`, `crates/fpas-lsp/src/formatting.rs`, `crates/fpas-cli/src/cli_fmt/mod.rs` | CLI and editor share `fpas_fmt::format_source`; malformed editor snapshots return no formatting result. Preserve common output and no destructive edits for invalid input. |
| `crates/fpas-debug/src/evaluation/parse.rs`, `target.rs` | Uses standalone expression parsing, including adjusted token streams for assignment targets. Operator/closer changes must also work outside compilation-unit parsing. |
| `.agents/skills/fpas-authoring/SKILL.md` and `references/examples.md`, `.agents/skills/fpas-projects/SKILL.md` | Current-language skeletons, import guidance, and source workflows must migrate with executable behavior. No guidance rewrite in this audit. |

Tracked/discoverable FPAS source counts from `rg --files <root> -g '*.fpas'` at
the audit baseline: `lib/Std/` 111, `lib/api/Std/` 23, `apps/` 17, `examples/`
129, `tests/` 540, `editors/vscode/test/` 8, formatter `tests/golden/` 11.
These are file counts, not executable test counts. Rust-embedded programs occur
throughout the workspace, including test support, project/build/linker tests,
compiler/VM tests, CLI process fixtures, debugger tests, and native benchmarks.
Editor TypeScript tests also embed programs. Migration discovery must include
these strings and source generators, not just `.fpas` files.

Current specification owners are `docs/specs/grammar.ebnf` and the handbook
pages mapped in [stage 1](source-map.md#grammar-and-handbook-map). In particular,
preserve production references for `program`, `unit`, `uses_clause`, declaration
blocks, `statement_list`, `if_stmt`, `case_arm`, both for forms, `while_stmt`,
`repeat_stmt`, `closure_expr`, and `record_update`. Update
`docs/pascal/tools/fmt-style.md` with the formatter implementation, not before it.

## Existing tests and target gaps

Paths in the table are relative to the named crate. These are reusable tests
of **current** behavior, not acceptance evidence for the new syntax.

| Area | Existing positive / negative / edge coverage | Required target additions or replacements |
|---|---|---|
| Lexer (`fpas-lexer`) | `src/tests/keywords.rs`: keyword table, case variants, keyword-prefix identifiers and exact spans; `identifiers.rs`: invalid non-ASCII recovery; `token_positions.rs`: Unicode and CRLF; `comment_spans.rs` and `errors/` | New words in mixed case and beside punctuation; keyword-like substrings in identifiers/comments/strings; formerly reserved identifiers only when their owning removal lands. |
| Bodies and declarations (`fpas-parser`) | `src/tests/decl/program.rs`, `decl/unit/`, `decl/routines.rs`, `decl/types/`, `stmt/blocks.rs`, `stmt/loops.rs`, `stmt/conditionals/`, `expr/closures.rs` | Every implemented block-table row, matching/missing/wrong closers, empty versus `null;` bodies, explicit block scope, nested else-if versus elsif, one keyword per public/default declaration. |
| Separators and recovery (`fpas-parser`) | `src/tests/errors/statement_separators.rs` retains following statements; `recovery.rs` covers malformed declarations and case arms; `delimiters.rs` has valid arguments and invalid trailing delimiters; `synthetic_eof.rs`, `nesting.rs`, `trailing_input.rs` cover truncation/limits | Replace `statement_list_boundaries_do_not_require_separators`, which explicitly accepts missing final semicolons. Add terminators before every branch/closer, extra semicolons, nested mismatched closer recovery, and bounded EOF cases. |
| Expressions (`fpas-parser`) | `src/tests/expr/aggregates.rs` checks nonempty updates, malformed fields and nested updates; `expr/precedence.rs` and `errors/chained_comparison.rs` check tree grouping and rejected chains | `end with`, anonymous routines before comma/parenthesis, new logical precedence, mixed-chain rejection, parentheses, and preservation of following statements after expression errors. |
| Names (`fpas-sema`, project and editor consumers) | `fpas-sema/src/tests/integration/short_names.rs` checks short/full access, ambiguity and missing imports; `fpas-language-service/tests/navigation_resolution.rs` covers hierarchical/ambiguous owners | Replace old short-name expectations; add alias-only access, duplicate imports, case-insensitive collisions, nested alias shadowing, forward type references, initializer order, and unchanged visibility. |
| Formatter (`fpas-fmt`) | `tests/comment_regressions.rs`: nested routines, closures, branch blocks, headers, import comments, CR-only input; `api_regressions.rs`: source mismatch, UTF-8 midpoint and overflowing spans; `golden_output.rs`, `round_trip.rs`, `fuzz_light.rs` | New canonical goldens, comments before/between/after named closers and branches, multiple alias imports, normalized AST comparison, scope/evaluation preservation, and second-format identity. |
| Scaffolds / generated API | `fpas-cli/src/main_tests/init.rs`: formatted checkable program/library/workspace, invalid arguments and conflicts; `fpas-language-service/tests/intrinsic_std_api.rs`: registry surface, syntax and editor queries | Regenerated units and every scaffold must parse/check in target syntax; retain negative non-overwrite and invalid-input tests. |
| Editor and CLI | `fpas-lsp/tests/snippets.rs` expands all defaults and parse/format/reparses; `tests/formatting.rs` checks unsaved-buffer parity, comments/idempotence and malformed-input refusal; `fpas-cli/src/main_tests/fmt.rs` tests CLI formatting/check/stdout behavior | Update snippet host wrappers as well as snippets; add target alias/closer parity and invalid partial-edit cases. Snippet parsing alone does not prove semantic validity. |
| Auto-import / highlighting | `fpas-language-service/tests/intellisense.rs`, `semantic_tools.rs` cover unique imports, ambiguous/private candidates, stale diagnostics and completion inside comments/strings; `editors/vscode/scripts/verify-grammar.mjs` executes `positive.fpas`, `negative.fpas`, `edge.fpas` token-scope assertions | Qualify inserted references and explicitly test preservation of commented import clauses; add alias keyword/symbol scopes, new closer/branch words, and identifier/string/comment negatives. Highlighting tests are not parser validation. |

Target runtime evaluation/visibility checks still belong to the implementation
slices and [shared verification](../verification.md). No future-language tests
are silently enabled, removed, or claimed passing by this inventory.

## Structure and next implementation boundary

Most parser concerns already have focused subdirectories. Before extending the
larger consumers, consider splitting `crates/fpas-fmt/src/comments/map.rs`
(403 lines), `comments/traversal.rs` (378), and `emit/decl/item.rs` (371) by
attachment, statement/declaration traversal, and routine/type emission as needed.
The API exporter is 433 lines with documentation already extracted; separate
source emission if that work would grow it further. The compiler expression
dispatcher is 490 lines; the existing sequence proposes a focused boolean
lowering module. These are implementation-boundary observations, not refactors
performed by this audit.

Next: the second stage-3 work item, using the
[syntax/resolution delivery sequence](implementation-sequence.md#syntax-and-resolution-deliveries).
Prepare exact implementation paths after choosing the coherent delivery boundary;
include AST, recovery, formatting/comments, consumers, current docs and tests.
No new language decision or competing target rule was introduced by this audit.

## Verification

Existing baseline checks run for this audit:

- `cargo test -p fpas-lexer -p fpas-parser -p fpas-fmt --quiet`: 557 passed,
  including three doc tests; none failed or ignored.
- `cargo test -p fpas-lsp --test snippets --test formatting --quiet`: 4 passed.
- `cargo test -p fpas-language-service --test intellisense --test intrinsic_std_api --test semantic_tools --quiet`:
  29 passed.
- `cargo test -p fpas-cli --bin fpas main_tests::init:: --quiet`: 8 passed.
- `cargo test -p fpas-cli --bin fpas main_tests::fmt:: --quiet`: 11 passed.
- `node editors/vscode/scripts/verify-grammar.mjs`: positive, negative and edge
  highlighting assertions passed.
- Relative links/heading anchors and fully specified repository paths in the
  three changed planning documents were checked; all resolved. `git diff --check`
  and the new audit file's whitespace check passed.

No new executable tests are needed for this documentation-only inventory.
The checks above validate the reusable baseline, not the target grammar. A full
workspace build/test or source-format rewrite is not an acceptance requirement
for this audit; the later language implementation still requires those checks.
