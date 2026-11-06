# Language-server platform acceptance

Only unresolved verification belongs here. Current behavior belongs in the
[language-server contract](../pascal/tools/language-server.md) and
[workspace architecture](../architecture/workspaces.md). Additional capabilities
remain in [editor gaps](editor-gap.md).

## Native Windows

Use native Windows with .NET 10, isolated package caches and the same reviewed
source revision for both configurations. Linux results, cross-compilation and
skipped platform cases do not establish this acceptance.

- [ ] Build `Fpas.slnx` in Debug and Release with `-warnaserror`.
- [ ] Run compiler tests and integration suites for Workspaces, Editing,
  Refactoring, Query, Analysis, Formatting, LanguageServer, Toolchain,
  LocalDeployment and documentation in both configurations. Record all skips.
- [ ] Verify independent stdio-client lifecycle, UTF-16/CRLF text changes,
  diagnostic replacement, navigation, search, versioned rename, formatting,
  linked-owner failures and CLI parity. Observe shutdown and EOF cleanup before
  harness disposal; add native coverage if a required process case is skipped.
- [ ] Run `LocalDeploymentTests` against `dist.ps1`, including both the published
  launcher and the SDK's extracted CLI with a fresh consumer outside the checkout.
  Verify passive `fpas env`, matching dependencies, no implicit restore/build or
  source/output writes, and clean process exit.

Keep TRX reports, environment/version manifests and bounded logs under ignored
`.temp-data/`; do not commit machine-specific paths or identities. Follow the
[test workflow](../../tests/README.md) and
[deployment instructions](../build/development.md#deploy-the-local-release-compiler).
Delete this document after the native acceptance is complete.
