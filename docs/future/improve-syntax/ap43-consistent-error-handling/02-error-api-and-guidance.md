# AP43.2: Error APIs and guidance

Package: [AP43](README.md). Status: blocked by AP43.1 and AP33.3.

## Scope and prerequisites

Implement the selected parsing/handling convention and complete the normative
guidance. AP35/AP39 implementation is required only for examples or API
removals relying on their new rules; otherwise use current behavior. Correcting
the try matrix alone does not authorize changing propagation semantics.

## Affected areas

Inspect `crates/fpas-std/src/conv/mod.rs`, `parse.rs`, `result_option.rs`, standard
registry signatures, generated declarations, and
`crates/fpas-sema/src/check/expr/mod.rs` for try compatibility. Update diagnostic
suggestions that currently teach a repeated predicate-plus-Unwrap access.

## Migration and documentation

Migrate aborting callers to explicit Unwrap only when preserving their chosen
failure behavior; recoverable callers retain Result handling. Update
[Conv](../../../pascal/std/text/conv.md), [Parse](../../../pascal/std/text/parse.md),
[try](../../../pascal/language/error-handling/try.md), Option/Result operations,
compact reference, diagnostics, and all affected consumers. Explain eager
argument evaluation rather than presenting lazy fallback as an equivalent
textual rewrite.

## Verification

Cover valid/invalid parsing, preserved abort versus returned-error behavior,
the full try type matrix, single payload evaluation, and effectful/panicking
fallbacks on both success and failure paths. Check current CLI examples,
runtime diagnostics, compiled units, and required implementation checks.
