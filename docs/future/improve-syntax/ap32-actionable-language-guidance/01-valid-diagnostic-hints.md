# AP32.1: Valid diagnostic hints

Package: [AP32](README.md). Status: pending correction.

## Scope and prerequisites

Repair the general ambiguous-import suggestion without removing valid
migration hints for actual native operations. AP02 and AP06 are complete.
Use qualified import candidates for ambiguous free calls; never suggest
receiver-based free-function lookup that the language does not implement.

## Affected areas

Inspect `crates/fpas-sema/src/check/name_resolution/std_names.rs`,
`crates/fpas-sema/src/tests/integration/short_names.rs`, and related diagnostic
owners. Distinguish the ambiguous-import path from removed-native-helper hints.

## Migration and documentation

No language migration. Synchronize the
[diagnostic reference](../../../pascal/tools/diagnostics.md) and examples that
quote affected messages. Preserve diagnostic codes where their meaning holds.

## Verification

Create a real ambiguous call such as imported `ReadText`; apply the suggested
qualification and check that it resolves correctly. Verify that equivalent
receiver syntax is not suggested for a free function. Retain tests showing
that native-operation migration hints still lead to valid code. Run affected
diagnostic/CLI tests and required workspace checks for Rust changes.
