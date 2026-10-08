# Documentation and specification findings

See [scope and verification](README.md). These findings concern current documentation and completed-package result text. Drafts in open packages are not treated as missing implementations. Each grouped migration finding lists its confirmed affected locations.

<a id="d01"></a>

## D01 — P2 — The AP13 documentation migration leaves invalid positive examples and corrections

AP13's accepted rule requires every statement/declaration to terminate with `;` and every case arm to start with `when`. Its completed README still asserts that all repository docs use the new syntax ([docs/future/improve-syntax/ap13-explicit-block-boundaries/README.md:191-196](../../../../docs/future/improve-syntax/ap13-explicit-block-boundaries/README.md)). The following current examples still violate it:

| Current file | Lines | Defect |
| --- | --- | --- |
| [docs/pascal/std/text/conv.md](../../../../docs/pascal/std/text/conv.md) | 94 | `WriteLn(BoolToStr(true))` lacks `;` before the following statement. |
| [docs/pascal/std/numeric/math.md](../../../../docs/pascal/std/numeric/math.md) | 261 | `WriteLn(Frac(3.14))` lacks `;`. |
| [docs/pascal/std/host/path.md](../../../../docs/pascal/std/host/path.md) | 81, 92, 108, 109, 114 | Five example call statements lack `;`. |
| [docs/pascal/std/console/types.md](../../../../docs/pascal/std/console/types.md) | 38, 145 | Both shown record definitions omit the last field's semicolon (`meta: boolean`). |
| [docs/pascal/std/console/screen-misc.md](../../../../docs/pascal/std/console/screen-misc.md) | 71 | `CursorOn()` lacks `;` before final `end.`. |
| [docs/pascal/language/types/record-methods.md](../../../../docs/pascal/language/types/record-methods.md) | 108 | The complete bound-method example omits `;` after `WriteLn(AddTen(5))`. |
| [docs/pascal/std/tui/application.md](../../../../docs/pascal/std/tui/application.md) | 20, 63 | `TuiMsg.QuitRequested:` and `TuiMsg.Started:` case-arm examples omit `when`. |
| [docs/pascal/tools/diagnostics.md](../../../../docs/pascal/tools/diagnostics.md) | 224, 227 | The suggested **corrections** for FP2003/FP2006 still omit the last statement's `;`. |
| [docs/pascal/tools/diagnostics.md](../../../../docs/pascal/tools/diagnostics.md) | 233, 234 | The suggested **corrections** for FP2012/FP2013 still close enums with `end;` instead of `end enum;`. |

An additional stale call-form example is [docs/pascal/std/console/output.md:28](../../../../docs/pascal/std/console/output.md): `WriteLn;` produces FP2006; the supported zero-argument form is `WriteLn();`.

The missing terminators/closers are parser failures (FP2001), not formatting differences. The code-fence parser pass rejects these snippets when wrapped in the necessary program/case context. Do not flag intentional API signature lists or incomplete context fragments merely for lacking a standalone entrypoint.

Needed coverage: extract compilable/parseable documentation examples and validate their syntax; include positive correction examples in the diagnostics catalog. Current Rust/fpas suites can pass while these Markdown literals are stale.

<a id="d02"></a>

## D02 — P2 — AP11 grouped declarations remain in current language and formatter documentation

- [docs/pascal/tools/fmt-style.md:464-478](../../../../docs/pascal/tools/fmt-style.md): `type Color = ...; Shape = ...;` still inherits the `type` keyword for the second type. `Shape` at line 471 requires its own `type`.
- [docs/pascal/language/types/record-events.md:26-29](../../../../docs/pascal/language/types/record-events.md): `type ClickHandler = ...; Button = record ...` repeats the same removed grouped-declaration form at line 29. Method headers in that fragment are schematic, but its declaration form is still wrong independently of those omitted bodies.
- Actual parser diagnostic is FP2015 for missing declaration keyword. Repeating `type` is required by the completed AP11.2 work package and current declarations grammar.
- Needed coverage: documentation migration scan/parse checks that include snippets, not just complete programs and `.fpas` files.

<a id="d03"></a>

## D03 — P2 — The formatter's complete control-flow example mutates a const binding

- [docs/pascal/tools/fmt-style.md:73](../../../../docs/pascal/tools/fmt-style.md) declares `const X: integer := 5;`, then line 115 executes `X := X + 1;`.
- Checking the complete claimed golden file (fence begins at line 67) fails FP3005: `Cannot assign to X`, with the `const`/`var` correction hint.
- This is a missed immutable/mutable migration in an example presented as a complete program. It needs `var X`, preserving the loop's behavior.
- Evidence: `.temp-data/syntax-review-syntax/docs/fmt-style-2.fpas` and `.temp-data/syntax-review-syntax/docs/results.json`.

<a id="d04"></a>

## D04 — P2 — The current formatter specification advertises unimplemented generic record types

- [docs/pascal/tools/fmt-style.md:478](../../../../docs/pascal/tools/fmt-style.md) presents `type IntBox = Box of integer;` as formatter output; line 485 lists `Box<T>`, `Box of string` and `Pair of integer, string` as supported generics.
- Current parser explicitly rejects user-defined generic type applications in [crates/fpas-parser/src/parser/decl/type_expr.rs:108-127](../../../../crates/fpas-parser/src/parser/decl/type_expr.rs) (FP2001); type declarations also do not accept user type parameters. AP24 remains open and its approved draft spelling is different.
- The current grammar permits `of` only for built-in generic forms, and [docs/pascal/language/types/generics.md](../../../../docs/pascal/language/types/generics.md) documents that boundary. Unimplemented syntax belongs only under [docs/future/](../../../../docs/future/).
- Expected: current formatter reference describes existing built-in generics and generic routines; keep AP24 examples in its plan until implemented.

<a id="d05"></a>

## D05 — P3 — The documented long-import golden is not formatter output

- [docs/pascal/tools/fmt-style.md:436-443](../../../../docs/pascal/tools/fmt-style.md) is explicitly labeled golden output but keeps the long `uses ...;` on one line at line 439.
- Running `fpas fmt --stdout .temp-data/syntax-review-syntax/fmt-golden-12.fpas` instead emits:

```pascal
uses
  Std.Console, Std.Conv, Std.Crypto, MyApp.Very.Long.Namespace.One, MyApp.Very.Long.Namespace.Two;
```

- The other complete program/unit fences in that formatter page match `fmt --stdout`. This example drift is separate from semantic errors in D03/D06 and parser errors in D02/D04.
- Needed coverage: compare documented canonical golden blocks against actual formatter output or reuse the tested golden sources in documentation generation.

<a id="d06"></a>

## D06 — P2 — Two complete introductory examples omit required Console imports

- [docs/pascal/getting-started/keywords.md:52-58](../../../../docs/pascal/getting-started/keywords.md) is a complete `PROGRAM KeywordDemo` using `writeln` without `uses Std.Console;`.
- [docs/pascal/tools/fmt-style.md:43-48](../../../../docs/pascal/tools/fmt-style.md) is a complete `program Hello` using `WriteLn` without the same import.
- Both fail FP3003, unknown procedure. Imports are required even for standard-library calls under the documented current import model. These are not isolated multi-file examples or expected negative cases.
- Evidence: `.temp-data/syntax-review-syntax/docs/keywords-0.fpas` and `.temp-data/syntax-review-syntax/docs/fmt-style-0.fpas` with `fpas check --std-lib lib`.

<a id="g01"></a>

## G01 — P2: The formal keyword set omits reserved discard

[docs/specs/grammar.ebnf:82-99](../../../../docs/specs/grammar.ebnf) omits `discard` from `keyword`, so its lexical grammar classifies it as an identifier. However, `discard_stmt` at line 361 requires it as a terminal and [crates/fpas-lexer/src/token/keywords.rs:16](../../../../crates/fpas-lexer/src/token/keywords.rs) reserves it. [docs/pascal/getting-started/keywords.md](../../../../docs/pascal/getting-started/keywords.md) correctly lists it. Add it to the formal keyword production and keep the lists mechanically consistent.

This is a specification discrepancy, not a request to change language behavior. Add keyword-set parity coverage.

<a id="g02"></a>

## G02 — P2: The formal task-call grammar excludes supported postfix targets

[docs/specs/grammar.ebnf:463-470](../../../../docs/specs/grammar.ebnf) says `go_call = designator '(' [ arg_list ] ')'` and explicitly describes that restriction. The implemented parser at [crates/fpas-parser/src/parser/stmt/mod.rs:102-119](../../../../crates/fpas-parser/src/parser/stmt/mod.rs) accepts a postfix expression ending in a method call. The valid current example `const Job: task := go Make().Answer();` cannot be derived from that production. `fpas run --std-lib lib .temp-data/syntax-review-syntax/go-postfix.fpas` prints `42` and exits 0. A grammar based on valid final-call expressions should reflect this existing behavior without broadening it to arbitrary expressions.

This is a specification discrepancy, not approval to broaden callable targets beyond current behavior. Add grammar conformance fixtures for normal and postfix task targets.

<a id="p01"></a>

## P01 — P3 — Completed package result text describes features already removed by other completed packages

- [docs/future/improve-syntax/ap10-typed-record-construction/01-typed-construction.md:37-40](../../../../docs/future/improve-syntax/ap10-typed-record-construction/01-typed-construction.md) states the contextual `record ... end` literal remains valid and its anonymous labels are still outside editor rename.
- [docs/future/improve-syntax/ap10-typed-record-construction/02-migrate-record-literals.md:9-10](../../../../docs/future/improve-syntax/ap10-typed-record-construction/02-migrate-record-literals.md) says the parser/compiler still accept the literal until AP10.3, although AP10.3 is marked complete and current parser emits FP2017.
- [docs/future/improve-syntax/ap06-dot-call-targets/README.md:11](../../../../docs/future/improve-syntax/ap06-dot-call-targets/README.md) says callable record properties remain actual member calls, although AP14 removed properties. Line 47 also discusses properties as rejected mutable receivers despite there being no current property declaration form.
- AP10.1 line 20 also lists properties among constructor non-fields; it should not imply a currently available member category.
- The central README says completed packages describe current implemented behavior. Remove the superseded intermediate-state claims and describe the final implemented forms. This does not imply that removed syntax should return; package checkbox changes are outside this report's scope.

<a id="t04"></a>

## T04 — P3: Runtime string bounds hints still recommend a removed free-call spelling

**Packages:** AP06 migration and AP02 actionable diagnostics.

**Requirement:** AP06 removes free type-helper calls and makes `S.Length()` the canonical string length form. AP02's cross-package rule requires canonical replacement spellings in diagnostics.

**Implementation:** [crates/fpas-std/src/str/mod.rs:195](../../../../crates/fpas-std/src/str/mod.rs) and `213` tell users to use `0..Length(S)-1`; `248` gives `0..Length(S)`. `Length(S)` no longer resolves to a native operation.

**Reproduction:**

```pascal
program Demo;
begin
  discard 'a'.CharAt(1);
end.
```

Run with `--diagnostics json`.

**Actual (confirmed):** `FP5021` includes `"hint":"Ensure the index is within 0..Length(S)-1."`. The same old spelling occurs for `SetCharAt` and `Insert` bounds errors.

**Expected:** Use the canonical `S.Length()` spelling in these hints, and ensure the empty-string case is described accurately when providing an interval.

**Impact:** Following the correction hint introduces a removed API spelling and triggers another semantic error. This is a diagnostic/documentation migration issue, not a string-indexing runtime defect.

**Tests needed:** Negative runtime native-operation tests that assert canonical helper spellings in hints. Do not blindly flag every private `Std.*` implementation ID: the confirmed problem here is user-directed replacement syntax.

**Evidence:** `.temp-data/syntax-review-tooling/catalog-results.json`, entries 44 and 45. Source owner lines above.

<a id="l01"></a>

## L01 — P3: Cryptography index links outside the documentation tree

[docs/pascal/std/cryptography/README.md:12](../../../../docs/pascal/std/cryptography/README.md) links to `../../../../future/networked-applications/cryptography.md`. That resolves to a nonexistent top-level `future/` directory. The existing target is [docs/future/networked-applications/cryptography.md](../../../../docs/future/networked-applications/cryptography.md), requiring `../../../future/networked-applications/cryptography.md` from the index.

This incidental documentation defect was found during the broad link sweep; it is not attributed to a syntax package. It was the only missing relative file target among 1,490 checked Markdown links. Correct the relative path and keep link validation in the documentation checks.


## Documentation validation coverage

The audit screened all 446 Pascal/fpas fences in the current handbook with parser-only checks in applicable synthetic contexts. Of 44 candidates, only manually confirmed errors are listed; API signature lists, schematic bodies and intentional incomplete snippets are not reported merely because they are not standalone programs.

Of 56 complete program/unit fences checked, 41 passed and 15 failed. Eleven failures need the documented sibling units or a valid unit/project context and are not findings. Four were substantive: the Greet compiler failure (C06), the two missing imports (D06), and the const mutation (D03). Formatter output was also compared for the complete formatter-reference fences; D05 is the mismatch. Interactive and network examples were checked without being launched.

The file-target scan covered 278 Markdown files and 1,490 relative links. All 887 literal Rust references to `docs/pascal/*.md` resolved to existing files. This does not validate every fragment anchor or external URL. No current specification, existing example or package checkbox was modified by this review.
