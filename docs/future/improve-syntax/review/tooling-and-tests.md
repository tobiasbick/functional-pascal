# Tooling, tests and documentation review

No active findings. This file is ready for the tooling portion of the **Spec**
axis of the [next review](README.md). Completed coverage is in
[reviewed-scope.md](reviewed-scope.md).

## Finding format

Use one heading per confirmed finding: `Tnn — Pn: concrete defect`.
Each finding must contain:

1. **Status and owner:** AP work package, affected service/tool/doc page and
   fixed reviewed revision.
2. **Requirement and reproduction:** the documented consumer contract, source
   fixture, public API or real command, query position where relevant, and
   actual versus expected result. Confirm whether the fixture is semantically
   valid or intentionally exercises diagnostics/recovery.
3. **Cause and bounded repair:** the shared resolution, presentation, transport
   or formatting owner; any affected aliases, scopes, imports and consumers.
4. **Acceptance:** relevant public behavior, isolation from unrelated symbols,
   valid edited sources, negative/boundary cases and applicable editor/CLI paths.
   Apply the repository's test-audit skill before changing tests.
5. **Closure evidence:** pre-repair failure, regression result, second review,
   documentation status and commit. Link checks from
   [verification.md](verification.md).

For documentation findings, cite the inconsistent page and implemented
behavior, check examples and links, and confirm the correction preserves the
agreed language. Keep consumer defects separate from independent Rust
standards findings. Record inspected areas and checks that were not run.
