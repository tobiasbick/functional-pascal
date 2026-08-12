# Variant transition assignment

Status: implemented and verified on 2026-08-12; evidence is tracked in
[`progress.md`](progress.md).

This package records textual debugger assignment that names an inactive variant
explicitly and provides the only payload value needed to construct it, for
example:

- `Optional.Some.value`;
- `Outcome.Ok.value` or `Outcome.Error.value`; and
- `Selected.Item.value` for an enum variant with exactly one declared field.

The shared VM debugger remains authoritative for JSONL, DAP, and VS Code.

## Start here

1. Read [`scope-and-decisions.md`](scope-and-decisions.md).
2. Review [`architecture.md`](architecture.md).
3. Remaining exclusions stay in [`consciously-deferred.md`](consciously-deferred.md).
4. Verification mapping lives in [`verification-matrix.md`](verification-matrix.md).
5. Resume notes and executed commands live in [`progress.md`](progress.md).

## Resume checkpoint

The implementation is present in the working tree on `codex/fpas-debugger`.
No source-language change is involved. If context is lost, inspect `git status`,
then resume at the first unchecked verification gate in `progress.md`; do not
recreate the runtime design from memory.
