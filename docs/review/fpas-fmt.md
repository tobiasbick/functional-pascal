# `fpas-fmt` review follow-up

Classification: formatter behavior. Fixes must preserve source semantics and every source comment. No language change expected.
Status: FMT-01 through FMT-06 completed 2026-08-03.

| ID | Priority | Evidence | Finding and impact | Implementation direction | Required regression |
| --- | --- | --- | --- | --- | --- |
| FMT-01 | P1 | `crates/fpas-fmt/src/comments/anchors.rs:107`, `src/emit/program.rs:41` | **Done.** The formatter globally selected the first `begin`; comments before routine bodies or the main body could attach to unconsumed anchors and disappear. | Body anchors are now collected structurally per program, routine, nested routine, record method, and closure owner. | Comments before routine `begin` and main `begin` when routines precede the program body; assert parseability and idempotence. |
| FMT-02 | P1 | `crates/fpas-fmt/src/emit/expr/closure.rs:36` | **Done.** Closure formatting used an empty `CommentMap`, and traversal omitted expressions and closure bodies. | The real comment map now flows through expression emission, while exhaustive expression traversal finds nested closure declarations and statements. | Leading/trailing/nested closure comments and repeated formatting. |
| FMT-03 | P1 | `crates/fpas-fmt/src/comments/anchors.rs:181`, `src/emit/decl/item.rs:142,219,238` | **Done.** Record member trailing comments were collected but not emitted; enum members and routine body endings were incomplete. | Shared declaration/routine finishers emit EOL comments; enum members participate in leading and trailing anchoring. Explicit block and final-program EOL comments also keep their separators on the code line. | EOL comments on fields, methods, properties, events, enum members, routines, blocks, and `end.`. |
| FMT-04 | P2 | `crates/fpas-fmt/src/emit/expr/literal.rs:65,77` | **Done.** Multiline array emission wrote a newline twice, adding blank lines before `]` and around multiline elements. | Padding-to-column and newline-to-column are distinct operations; width uses Unicode display columns. | Exact Unicode width boundaries and arrays containing multiline record/closure elements. |
| FMT-05 | P3 | `crates/fpas-fmt/src/emit/stmt/line.rs:26` | **Done.** Local variable statements deep-cloned the AST solely to recover a start offset. | Variable statements pass `var.span.offset` to an offset-based line finisher. | Immutable/mutable variables in last/non-last positions with EOL comments. |
| FMT-06 | P3 | `crates/fpas-fmt/src/lib.rs:27`, `src/comments/anchors.rs:341` | **Done.** `format_source` could panic or silently misassociate comments when `source` did not match `unit`. | The public API returns `Result<String, FormatError>`, validates every consumed span, and rejects a source/AST mismatch. CLI and editor callers handle the fallible result. | Same-length mismatches, invalid UTF-8 boundaries, and overflowing/out-of-range spans. |

## Implementation notes

Comment preservation is the primary contract; add regressions before changing anchoring. Include CR-only input, combining/wide Unicode near the width boundary, and EOL comments following explicit block bodies. When `.fpas` fixtures are added, run the repository FPAS formatter check as required by `AGENTS.md`.

## Implementation record

- Comment attachment was split into safe span/line primitives and recursive AST traversal; no source-text keyword search is used to assign a body to an owner.
- Round-trip helpers now compare normalized comment sequences, so parseability and idempotence cannot hide comment deletion.
- Bare `CR` input is normalized to `LF`; combining and wide Unicode characters are measured in display columns.
- Documentation updated: `docs/pascal/tools/fmt-style.md`. No FPAS syntax or semantic change.
- `format_source` intentionally changed directly to `Result<String, FormatError>` instead of retaining a compatibility wrapper. This repository has no backward-compatibility requirement, and all workspace callers now handle the fallible boundary.

Verification:

- `cargo test -p fpas-fmt`: 83 tests passed, including repository-source round trips.
- `cargo check --workspace`: passed.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`: passed.
- `cargo fmt --all -- --check`: passed.
- `cargo build --workspace`: passed.
- `cargo test --workspace`: passed in a clean rerun. An earlier bounded run timed out and its immediate retry encountered a transient Windows linker lock; neither reported a code or test failure.
