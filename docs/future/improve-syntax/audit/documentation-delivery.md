# Stage 3 coordinated documentation delivery

Scope: the grammar, handbook, formatter style, authoring guidance, source
templates, and editor-formatting item in [stage 3](../stages/03-syntax-and-names.md).
The [block/name](block-syntax-delivery.md), [operator/bit API](operator-delivery.md),
and [source-conversion](source-conversion.md) deliveries already implemented and
migrated their constructs. This delivery reconciles their documentation and
verifies the actual consumers. It adds no language rule or standard-library API.

## Corrections and confirmed consumers

- `docs/specs/grammar.ebnf` describes nonempty statement bodies while allowing
  empty declaration-only units. Its record-update example uses `end with`.
  The keyword production matches the lexer; the handbook keyword table now
  includes the previously omitted `as`, `elsif`, `when`, and `null`.
- Routine-declaration schemas use `end function;` and `end procedure;`. Case
  schemas use `when`, statement lists, mandatory terminators, and `end case;`.
  The case example is a runnable program with distinct branch-local bindings.
  Binding guidance requires one keyword per declaration, and routine guidance
  requires an individual type annotation for each formal parameter.
- Closure guidance distinguishes expression closers from surrounding argument
  separators. A runnable example passes anonymous functions and procedures
  directly before a comma and a closing parenthesis.
- Early-return, nested-routine, Option, and task-handle examples explicitly
  import and qualify their array, math, and task operations. Record-update and
  event examples use their implemented closers. Dictionary iteration guidance
  distinguishes iteration/indexing from explicitly imported library calls.
- Formatter style describes the blank line before a unit closer. Authoring
  skeletons match canonical output. Project-authoring guidance includes aliases
  and qualified uses of exported routines.
- Editor documentation explains canonical named closers, semicolons, import
  aliases, preserved explicit blocks and nested conditionals, and no edits for
  malformed input. The existing editor and CLI share `fpas-fmt`.
- CLI project, library, and workspace templates already use canonical syntax.
  Existing init tests verify their formatting and project checking, including
  invalid inputs, idempotency, conflicts, dry runs, and linked workspaces.
  Existing VS Code snippet assets likewise require no source rewrite; their
  parser/formatter tests now also check second-format identity and rejection of
  obsolete forms. Intrinsic API regeneration leaves `lib/api/Std/` unchanged.

## Formatter regression discovered during verification

The new empty-unit edge case reproduced a comment-attachment failure with
`unit Empty; end unit; // tail`. The formatter attached `// tail` to the unit
header; formatting that output again moved it a second time. CRLF and Unicode
were unnecessary to reproduce it.

`crates/fpas-fmt/src/comments/traversal.rs` formerly selected the last semicolon
before a unit's first import/declaration, falling back to its complete span.
For an empty unit that selected the new `end unit;` terminator as the header
anchor. It now selects the unit header's first semicolon. The existing comment
emitter and span validation remain authoritative.

`crates/fpas-fmt/tests/comment_regressions.rs` locks down the original failure
with exact output and structural round trips, plus separate header/closer
comments, CRLF, and import-only units. The regression was observed failing
before the fix and passing afterward. LSP tests exercise the same edge case
through the editor protocol.

## Positive, negative, and edge-case coverage

Focused `crates/fpas-cli/src/main_tests/handbook/` modules read actual Markdown
instead of maintaining parallel copies of its examples:

- All 60 complete handbook programs/units parse, format, reparse, retain every
  comment, and produce identical second formatting. Three formatter examples
  and three authoring skeletons additionally match exact canonical output.
- Runnable case and callback examples retain their output before and after
  in-place CLI formatting. Documented routines cover empty collections, missing
  matches, last-element matches, and zero inputs. Task examples verify qualified
  waits, result-bearing handles, Result-valued task arrays, and barriers.
- Invalid derivatives reject missing/mismatched closers, missing/extra
  terminators, empty bodies, alias-free imports, and semicolons inserted before
  callback argument separators. Failed formatting leaves the source unchanged.
- Semantic checks reject unqualified imported calls, case-insensitive alias
  collisions, and branch-local names used after a case, with specific codes and
  messages. Checking leaves the source unchanged.
- Keyword-table tests compare handbook and EBNF words against lexer tokens,
  including case variants, retired identifiers, and keyword-prefixed names.
- Fence extraction preserves source bytes across CRLF, ignores non-Pascal
  fences, and handles a final fence without a newline. Formatting edge cases
  include empty units and Unicode comments containing obsolete spellings.

`crates/fpas-lsp/tests/formatting.rs` verifies actual formatter-handbook output
through unsaved buffers, including CRLF and noncanonical editor tab preferences.
The second request produces no edit. Malformed closers, terminators, and imports
also produce no edit. `tests/snippets.rs` covers every existing snippet, obsolete
closers/imports with migration hints, case variants, CRLF, and retained comments.

## Verification

- `cargo fmt`, `cargo fmt --all --check`, and `cargo build --quiet` pass.
- `cargo test --workspace --quiet` passes: 3,442 tests across 187 test/doc-test
  suites, zero failures and zero ignored tests. The delivery adds 17 tests:
  11 handbook tests, two formatter regressions, and four LSP/snippet tests.
- Targeted handbook tests pass; formatter comment regressions pass all 13 tests,
  and LSP formatting/snippet suites pass all nine tests.
- `fpas fmt --check lib examples tests apps` passes all 822 source files.
- `fpas test tests/suite.fpasprj --jobs 8`: 459 passed, one intentional skip,
  zero failures.
- `cargo run -p fpas-sema --example export_intrinsic_std_api --quiet` passes
  without changing generated declarations.
- `node editors/vscode/scripts/verify-grammar.mjs` passes.
- `node editors/vscode/scripts/run-tests.mjs` passes the full real extension-host
  suite, including diagnostics, formatting, navigation, IntelliSense, semantic
  tools, project workflows, debugger, and lifecycle checks, on its isolated
  repeat. The first run, concurrent with workspace tests, failed a debugger
  copied-task-handle output assertion because a session-terminated message was
  appended. No debugger implementation or assertion was changed for the repeat.
- Relative links in the changed Markdown and Rust documentation paths resolve.
  `git diff --check` passes. Repository documentation contains no machine-specific
  metadata.

## Remaining stage work

The coordinated documentation item is complete. The
[full coverage delivery](coverage-delivery.md) verifies the final checklist item
and closes stage 3 for implemented constructs. Conditional/case expressions, task
scopes, replacement binding syntax, and removal of methods/properties/events
remain with their owning later stages; the current handbook continues to
describe their existing implementations.
