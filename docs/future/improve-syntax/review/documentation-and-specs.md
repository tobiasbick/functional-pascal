# Documentation and specification findings

See [scope and verification](README.md). These findings concern current documentation and completed-package result text. Drafts in open packages are not treated as missing implementations. Each grouped migration finding lists its confirmed affected locations.

Finding-specific test results below describe the original local repairs. The
[local/cloud reconciliation](README.md#localcloud-reconciliation) records checks
against the combined checkout.

<a id="d01"></a>

## D01 — P2 — AP13 documentation examples and corrections (resolved)

**Status: resolved.** The affected handbook examples use the implemented AP13
terminators, named closers, and `when` case arms. This repair changes documentation
and regression coverage; language, parser, runtime, and standard-library behavior
are unchanged.

| Current file | Corrected examples |
| --- | --- |
| [docs/pascal/std/text/conv.md](../../../../docs/pascal/std/text/conv.md) | The Boolean conversion call ends with `;`. |
| [docs/pascal/std/numeric/math.md](../../../../docs/pascal/std/numeric/math.md) | Both fractional-part calls end with `;`. |
| [docs/pascal/std/host/path.md](../../../../docs/pascal/std/host/path.md) | Every example call statement ends with `;`. |
| [docs/pascal/std/console/types.md](../../../../docs/pascal/std/console/types.md) | Both conceptual records terminate their final `meta` field. |
| [docs/pascal/std/console/screen-misc.md](../../../../docs/pascal/std/console/screen-misc.md) | `CursorOn();` terminates before `end.`. |
| [docs/pascal/language/types/record-methods.md](../../../../docs/pascal/language/types/record-methods.md) | The bound-method example terminates `WriteLn(AddTen(5));`. |
| [docs/pascal/std/tui/application.md](../../../../docs/pascal/std/tui/application.md) | The quit and startup case arms begin with `when`. |
| [docs/pascal/tools/diagnostics.md](../../../../docs/pascal/tools/diagnostics.md) | FP2003/FP2006 corrections terminate statements; FP2004 uses a typed loop variable and `end for;`; FP2012/FP2013 use `end enum;`. Negative examples isolate their catalogued error, including an invalid statement start for FP2003. |
| [docs/pascal/std/console/output.md](../../../../docs/pascal/std/console/output.md) | The zero-argument call is `WriteLn();`. |

Regression ownership:

- [Handbook examples](../../../../crates/fpas-parser/tests/documentation/examples.rs)
  extract Pascal fences directly from the affected pages and parse them with the
  necessary program, declaration, statement, or case context. Coverage includes
  both final record fields, the last calls before `end.`, and the TUI tick arm
  alongside quit and startup. Schematic API signatures are excluded.
- [Diagnostic examples](../../../../crates/fpas-parser/tests/documentation/corrections.rs)
  extract catalog pairs for FP2001–FP2006 and FP2012/FP2013. Positive corrections
  must parse without errors; negative examples must emit their catalogued code.
- [Markdown extraction](../../../../crates/fpas-parser/tests/documentation/markdown.rs)
  preserves source comments and line numbers and ignores non-Pascal fences.

<a id="d02"></a>

## D02 — P2 — AP11 documentation uses individual type declarations (resolved)

**Status: resolved.** Both affected handbook snippets repeat `type` on the
second declaration, following the implemented AP11.2 rule.

- [Formatter reference](../../../../docs/pascal/tools/fmt-style.md): `Shape`
  starts with `type`, independently of the preceding `Color` declaration.
- [Record events](../../../../docs/pascal/language/types/record-events.md):
  `Button` starts with `type`, independently of the preceding `ClickHandler`.
- [Handbook declaration tests](../../../../crates/fpas-parser/tests/documentation/declarations.rs)
  extract both examples from Markdown. Positive cases parse without errors and
  retain every declaration; removing the second keyword reproduces exactly one
  FP2015 diagnostic. The schematic event accessor signatures receive bodies in
  the test context. The formatter check covers the complete type snippet,
  including its built-in option alias after D04.

Language and parser behavior are unchanged. Regression coverage is also recorded
in [AP11.2](../ap11-individual-declarations/02-one-keyword-per-declaration.md).

<a id="d03"></a>

## D03 — P2 — The formatter control-flow example uses a mutable loop binding (resolved)

**Status: resolved.** The complete `ControlFlowDemo` program in the
[formatter reference](../../../../docs/pascal/tools/fmt-style.md) declares
`var X: integer := 5;`, permitting the documented `X := X + 1;` loop update.
The initial value and control-flow behavior are preserved.

The [formatter handbook tests](../../../../crates/fpas-cli/src/main_tests/examples/formatter_handbook.rs)
extract the complete program directly from Markdown and verify:

- `fpas check` accepts the complete example.
- `fpas run` terminates with the expected branch and loop output.
- `fpas fmt --stdout` matches the documented canonical block.
- Restoring `const X` produces exactly one FP3005 diagnostic during checking;
  the invalid program is not executed.

[AP16.3](../ap16-immutable-and-mutable-bindings/03-keyword-switch.md) records this
regression coverage. Language, compiler, and runtime behavior are unchanged.

<a id="d04"></a>

## D04 — P2 — The formatter documents implemented generic forms (resolved)

**Status: resolved.** The [formatter reference](../../../../docs/pascal/tools/fmt-style.md)
uses `type IntOption = option of integer;` in its complete type snippet, which
now matches canonical formatter output. The summary lists the six implemented
built-in generic forms and a generic routine signature with inferred call
arguments, linked to the current [Generics reference](../../../../docs/pascal/language/types/generics.md).

Regression ownership:

- [Handbook declaration tests](../../../../crates/fpas-parser/tests/documentation/declarations.rs)
  parse the entire type snippet, including the option alias; no declaration is
  excluded from the check.
- [Generic spelling tests](../../../../crates/fpas-parser/tests/documentation/generics.rs)
  parse all six forms extracted from the summary. Restoring `Box of integer`,
  `Box of string`, or `Pair of integer, string` in the alias produces exactly
  one FP2001 diagnostic for each unsupported application.
- [CLI handbook tests](../../../../crates/fpas-cli/src/main_tests/examples/formatter_generics.rs)
  extract the type snippet and routine signature directly from Markdown. With
  the necessary program and routine-body context, `fpas check` accepts them,
  `fpas run` verifies integer and string argument inference, and
  `fpas fmt --stdout` matches the documented type snippet.

This repair changes documentation and regression coverage only. Language,
parser, formatter, compiler, and runtime behavior are unchanged.
[AP24](../ap24-generic-data-structures/README.md) remains open; its draft examples
stay in the future plan.

<a id="d05"></a>

## D05 — P3 — The documented long-import golden matches formatter output (resolved)

**Status: resolved.** The complete `LongUses` program in the
[formatter reference](../../../../docs/pascal/tools/fmt-style.md) matches the
existing [formatter golden](../../../../crates/fpas-fmt/tests/golden/long_uses.expected.fpas).
The wrapping table and example explanation describe the implemented rule:
when the complete import clause exceeds 100 columns, `uses` occupies its own
line and the import list is indented by two spaces. Additional list wrapping
occurs after commas when needed.

The [CLI handbook tests](../../../../crates/fpas-cli/src/main_tests/fmt/handbook_imports.rs)
extract the complete program directly from Markdown and verify:

- The handbook block agrees with the existing golden and `fpas fmt --stdout`.
- `fpas fmt --check` accepts the documented block without changing the file.
- Restoring the original single-line clause returns the formatter's
  would-change exit code without changing the file; formatting that source
  produces exactly the documented output.

This repair changes documentation and regression coverage only. Formatter,
parser, compiler, and runtime behavior are unchanged.

<a id="d06"></a>

## D06 — P2 — Complete introductory examples import Console (resolved)

**Status: resolved.** Both complete programs import `Std.Console` before their
output call:

- The [keyword example](../../../../docs/pascal/getting-started/keywords.md)
  uses `USES Std.Console;`, preserving its mixed keyword and identifier casing.
- The [minimal formatter program](../../../../docs/pascal/tools/fmt-style.md)
  uses canonical `uses Std.Console;` and retains its hello-world output.

The [CLI handbook tests](../../../../crates/fpas-cli/src/main_tests/examples/introductory_handbook.rs)
extract both complete programs directly from Markdown and verify:

- `fpas check` accepts both examples.
- `fpas run` prints their documented messages, including the case-insensitive
  keyword and procedure names in `KeywordDemo`.
- Removing the import from either example produces exactly one FP3003
  diagnostic for the unknown procedure; negative cases are checked only.
- `fpas fmt --stdout` matches the complete minimal formatter program.

This repair changes documentation and regression coverage only. The explicit
[unit import model](../../../../docs/pascal/program-structure/units.md#using-units),
compiler, and runtime behavior are unchanged.

<a id="g01"></a>

## G01 — P2 — The formal keyword set matches the implemented reserved words (resolved)

**Status: resolved.** The `keyword` production in the
[formal grammar](../../../../docs/specs/grammar.ebnf) includes `discard`, agreeing
with the existing lexer and [handbook table](../../../../docs/pascal/getting-started/keywords.md).
The statement production and implemented keyword behavior are unchanged.

The [keyword documentation tests](../../../../crates/fpas-lexer/src/tests/keywords/documentation.rs)
extract spellings directly from the existing
[lexer mapping](../../../../crates/fpas-lexer/src/token/keywords.rs), the EBNF
production, and the handbook table. They verify:

- Both documented sets match all 65 implemented reserved words, reporting
  missing or unexpected entries; empty and duplicate inventories fail too.
- Every keyword is tokenized as reserved in lowercase, uppercase, and mixed case.
- `discard` remains reserved after a dot, while longer identifiers, strings,
  and comments preserve their existing token behavior.

Before the correction, the parity test reported exactly the missing `discard`
entry and no unexpected words. This repair aligns the syntax annex with the
current language; lexer, parser, compiler, and runtime behavior are unchanged.

<a id="g02"></a>

## G02 — P2 — The formal task-call grammar includes supported postfix targets (resolved)

**Status: resolved.** The `go_call` production in the
[formal grammar](../../../../docs/specs/grammar.ebnf) describes direct calls,
the existing keyword-owned array factory call form, and postfix chains whose
final suffix is a call. It reuses `call_args`, `primary_atom`, and `postfix_suffix`,
so `const Job: task := go Make().Answer();` and intervening field/index/method
suffixes are represented. The accompanying constraint still requires the
expression parsed after `go` to have a call as its outermost shape. The
[task-call reference](../../../../docs/pascal/language/concurrency/go.md) and
implemented language behavior are unchanged.

The [parser fixtures](../../../../crates/fpas-parser/tests/documentation/task_calls.rs)
and [CLI/VM regressions](../../../../crates/fpas-cli/src/main_tests/examples/concurrency/task_calls.rs)
verify:

- The formal production retains direct-call alternatives and requires a call
  as the final postfix suffix, rather than accepting an arbitrary expression.
- Both retained and detached forms parse ordinary, qualified, indexed, and
  callable-variable targets, as well as final calls after factories, constructors,
  fields, indexes, earlier methods, and parenthesized primary expressions.
- Bare values, final fields/indexes, outer operators, record updates, and a
  parenthesized whole call produce exactly one FP2005; parser recovery preserves
  the following statement.
- Eleven ordinary and postfix targets pass `fpas check` and return `42` through
  `Wait` when run. A detached postfix function delivers its side effect through
  a channel.
- A direct record constructor and a final call on a non-callable field still
  produce FP3006 during CLI checking.

Before the correction, the formal-production regression failed while all three
parser fixture groups passed. This repair changes the syntax annex and adds
nine regression tests; parser, semantic checker, compiler, and runtime are unchanged.

**Separate follow-up observed during validation:** for a declared record procedure
`Report`, the detached statement `go Make().Report();` parses but produces FP3006
(the procedure does not return a value). The
[statement checker](../../../../crates/fpas-sema/src/check/stmt/mod.rs) checks this
postfix target as a value expression. This existing semantic limitation is outside
the G02 grammar correction and remains open.

<a id="p01"></a>

## P01 — P3 — Completed package text describes the final implemented forms (resolved)

**Status: resolved.** The completed-package pages now agree with the implemented
AP10.3 and AP14 removals:

- [AP10.1](../../../../docs/future/improve-syntax/ap10-typed-record-construction/01-typed-construction.md)
  describes typed construction as the only record-construction form and links
  the completed migration and removal packages. The old expression form reports
  FP2017 with a typed-construction hint. The obsolete anonymous-label editor
  claim and property member category are removed; the sema owner describes
  named-field validation and construction metadata.
- [AP10.2](../../../../docs/future/improve-syntax/ap10-typed-record-construction/02-migrate-record-literals.md)
  records the completed AP10.3 removal after migration, rather than claiming an
  active compatibility window for record literals.
- [AP06](../../../../docs/future/improve-syntax/ap06-dot-call-targets/README.md)
  describes callable record fields and declared accessor methods, with a link
  to the completed AP14 removal. Its writable-receiver rules reject dictionary
  entries and computed receivers without implying that property declarations
  remain available.

Existing regression coverage was checked:

- Nine [removed-literal](../../../../crates/fpas-parser/src/tests/errors/removed_record_literals.rs)
  and [removed-property](../../../../crates/fpas-parser/src/tests/errors/removed_properties.rs)
  parser tests pass, including replacement hints, ordinary `property` identifiers,
  empty/nested literal positions, and recovery of subsequent statements/members.
- Two [writable-receiver tests](../../../../crates/fpas-sema/src/tests/expr/var_parameters/mutating_intrinsics.rs)
  pass for writable fields/elements and forwarded parameters, and rejection of
  read-only, dictionary, and computed storage.
- The [compiler/VM callable-field test](../../../../crates/fpas-compiler/src/tests/aggregates/record_construction.rs)
  passes for callable defaults and explicitly supplied callable fields.
- The [typed-construction FPAS test](../../../../tests/stdlib/records/typed_construction_test.fpas)
  passes through `fpas test --std-lib lib tests/stdlib/records/typed_construction_test.fpas`.

This is a documentation-only correction. No new tests or language changes are
needed, and the package-completion checkboxes are unchanged.

<a id="t04"></a>

## T04 — P3 — String bounds hints use native calls and handle empty strings (resolved)

**Packages:** AP06 migration and AP02 actionable diagnostics.

**Status: resolved.** The [string runtime](../../../../crates/fpas-std/src/str/mod.rs)
uses the native call spellings in FP5021 hints:

- `CharAt` and `SetCharAt` name `0..S.Length()-1` for nonempty strings.
- For empty strings, both operations report that no character index is valid
  and recommend checking `S.IsEmpty()`, without suggesting an invalid interval.
- `Insert` names the inclusive `0..S.Length()` range. Index `0` remains valid
  for an empty string, and the end position remains valid for nonempty strings.

The character operations share their hint selection. Error codes, messages,
source locations, argument validation, and valid scalar-index behavior are
unchanged. Private intrinsic IDs and callable targets are unchanged.

The [runtime tests](../../../../crates/fpas-std/src/str/tests/index_diagnostics.rs)
and [CLI diagnostics tests](../../../../crates/fpas-cli/src/main_tests/diagnostics/string_bounds.rs)
add ten regression tests covering:

- Negative, first-invalid, and extreme integer indices; ASCII, Unicode scalar
  counts, empty strings, valid first/last character indices, and start/end
  insertion positions.
- Exact FP5021 hints directly from the runtime and through text/JSON CLI
  output, with runtime exit status, source file/line, and empty stdout.
- Real `fpas check` and `fpas run` use of `S.Length()` and `S.IsEmpty()` in
  corrected programs, including guarded empty-string access and insertion.

Before the correction, three runtime and four CLI tests failed on the old hint
text while valid-boundary tests passed. All 23 string runtime tests and five
new CLI tests now pass, as does the full workspace suite (3827 passed, zero
failed or ignored).

The [character reference](../../../../docs/pascal/language/types/string/format-chars.md),
[editing reference](../../../../docs/pascal/language/types/string/edit.md), and
[FP5021 catalog entry](../../../../docs/pascal/tools/diagnostics.md) describe the
implemented bounds and hints. Formatting, build, Clippy, Rust documentation
targets, and relative Markdown links pass. The editor declaration exporter runs
successfully and produces no content changes.

<a id="l01"></a>

## L01 — P3 — Cryptography index resolves the future-work link (resolved)

**Status: resolved.**
[docs/pascal/std/cryptography/README.md:12](../../../../docs/pascal/std/cryptography/README.md)
uses `../../../future/networked-applications/cryptography.md`, which resolves to
the existing
[docs/future/networked-applications/cryptography.md](../../../../docs/future/networked-applications/cryptography.md).
The previous path traversed one directory too far upward and targeted a
nonexistent top-level `future/` directory.

This incidental documentation defect was found during the broad link sweep; its
origin is not attributed to a syntax package. It was the only missing relative
file target among the 1,490 links checked during the original audit.

**Verification:** The repair-time scan reproduces L01 as the only missing target
across 283 Markdown files and 1,651 relative links before the correction. After
the correction, all 1,651 relative links and 889 literal Rust handbook references
resolve. A direct path check confirms the exact intended planning document, and
`git diff --check` passes. This documentation-only repair requires no new Rust or
FPAS tests; link validation is the applicable check.


## Documentation validation coverage

The audit screened all 446 Pascal/fpas fences in the current handbook with parser-only checks in applicable synthetic contexts. Of 44 candidates, only manually confirmed errors are listed; API signature lists, schematic bodies and intentional incomplete snippets are not reported merely because they are not standalone programs.

Of 56 complete program/unit fences checked, 41 passed and 15 failed. Eleven failures need the documented sibling units or a valid unit/project context and are not findings. Four were substantive: the Greet compiler failure (C06), the two missing imports (D06), and the const mutation (D03). Formatter output was also compared for the complete formatter-reference fences; D05 is the mismatch. Interactive and network examples were checked without being launched.

The file-target scan covered 278 Markdown files and 1,490 relative links. All 887 literal Rust references to `docs/pascal/*.md` resolved to existing files. This does not validate every fragment anchor or external URL. No current specification, existing example or package checkbox was modified by this review.
