# Review verification

No active review checks or repair results are recorded. This file is ready for
the [next review](README.md). Completed AP coverage remains in
[reviewed-scope.md](reviewed-scope.md); earlier check results remain in Git history.

## Required record

Record the baseline, target and checked revision, selected work packages,
commands actually run, results and coverage limits. Use repository-relative
paths and omit machine-identifying metadata.

During a review, add a check table with the columns `Command or check`,
`Checked revision/scope` and `Result`. Distinguish passed, failed and not run;
do not carry earlier green results forward as evidence for a new revision.

Select checks according to [AGENTS.md](../../../../AGENTS.md) and the affected
behavior:

- Rust changes normally require `cargo fmt --all`, `cargo build` and
  `cargo test --workspace`; document a justified narrower scope.
- FPAS test changes require the owning runner tests and source-format checks.
- Tooling changes require applicable language-service, LSP, formatter, CLI or
  VS Code checks at the real consumer boundary.
- Documentation-only changes require link, command and example validation.
- Run applicable additional checks, such as crate-scoped Clippy, when required
  by the affected area. Record environmental failures and corrected reruns.

## Per-finding evidence

Add a table with `Finding`, `Pre-repair failure`, `Post-repair checks`,
`Second review` and `Commit`. Explain why each regression failed before repair
and how its assertions establish the intended public behavior afterward.

Record the second review's conclusion and any corrections it required before
marking an item repaired and verified. Finish applicable checks, update the
plan and commit each finding before proceeding to the next one.

## Coverage limits

List omitted checks, unresolved observations outside the selected scope and
any limits of the probes. Distinguish review completion from merge approval and
from claims of exhaustive coverage.
