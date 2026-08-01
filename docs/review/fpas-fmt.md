# `fpas-fmt` review follow-up

Classification: formatter behavior. Fixes must preserve source semantics and every source comment. No language change expected.
Status: all findings open.

| ID | Priority | Evidence | Finding and impact | Implementation direction | Required regression |
| --- | --- | --- | --- | --- | --- |
| FMT-01 | P1 | `crates/fpas-fmt/src/comments/anchors.rs:107`, `src/emit/program.rs:41` | The formatter globally selects the first `begin`; comments before routine bodies or the main body can be attached to unconsumed anchors and disappear. | Associate keyword anchors structurally with each program/routine body and emit them at that body. | Comments before routine `begin` and main `begin` when routines precede the program body; assert parseability and idempotence. |
| FMT-02 | P1 | `crates/fpas-fmt/src/emit/expr/closure.rs:36` | Closure formatting deliberately uses an empty `CommentMap`; traversal also omits expression/closure bodies. Comments inside closures are lost or moved. | Pass the real comment map through expression emission and recursively anchor closure declarations/statements. | Leading/trailing/nested closure comments and repeated formatting. |
| FMT-03 | P1 | `crates/fpas-fmt/src/comments/anchors.rs:181`, `src/emit/decl/item.rs:142,219,238` | Record field/property/event trailing comments are collected but never emitted; enum members are not anchored. | Use a shared member-line finisher that emits trailing comments and include enum members in collection/emission. | EOL comments on fields, methods, properties, events, and enum members. |
| FMT-04 | P2 | `crates/fpas-fmt/src/emit/expr/literal.rs:65,77` | Multiline array emission writes a newline twice, adding blank lines before `]` and around multiline elements. | Separate indentation-to-column from newline emission and call the correct primitive based on current line state. | Golden output near the width limit and arrays containing multiline record/closure elements. |
| FMT-05 | P3 | `crates/fpas-fmt/src/emit/stmt/line.rs:26` | Local variable statements deep-clone the AST solely to recover a start offset. | Pass `var.span.offset` directly to a finisher that accepts an anchor position. | Existing output remains identical; a focused unit test can protect the refactor. |
| FMT-06 | P3 | `crates/fpas-fmt/src/lib.rs:27`, `src/comments/anchors.rs:341` | Lower confidence: `format_source` can panic when `source` does not match `unit`, but the precondition is undocumented. | Either validate slices with `source.get` and return an error, or explicitly document source/AST identity and `# Panics`. Prefer a fallible public boundary. | Mismatched source, invalid UTF-8 boundary spans, and out-of-range spans. |

## Implementation notes

Comment preservation is the primary contract; add regressions before changing anchoring. Include CR-only input, combining/wide Unicode near the width boundary, and EOL comments following explicit block bodies. When `.fpas` fixtures are added, run the repository FPAS formatter check as required by `AGENTS.md`.
