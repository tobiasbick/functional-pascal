# C# review gaps

Open findings from the repository-wide C# review (excluding `apps/fpas-amp`
and `third-party/`). Remove entries once they are resolved.

## Documentation

- [ ] Document the public declarations in `src/Fpas.Workspaces` (about 300
  across roughly 45 files) as required by AGENTS.md.

## Compiler

- [ ] `shl`/`shr` emit no explicit shift-count mask, unlike C#. Results match on
  x64 and ARM64 today; decide whether FPAS defines masking and emit it explicitly.
- [ ] The diagnostic bag reports 49 errors plus the limit notice while its help
  text promises 50. Align the limit and the text.
- [ ] Native-integer constant folding uses the host pointer size instead of the
  target platform.

## Toolchain

- [ ] `WorkspaceLoader` recomputes the compiler hash for every project in its
  loading loop; compute it once.
- [ ] `LiveWorkspace` filters unowned documents twice.
- [ ] `FpasCompile` never disposes its cancellation source.

## Tests

- [ ] `GuiHostTests` packs the SDK separately for each renderer instead of
  sharing `GuiPackageFixture`.
- [ ] `tests/Fpas.IntegrationTests` still computes SHA-256 file hashes inline
  (for example in `Workspaces/CheckTests.cs`); use `TestSupport/FileHash`.

## Verification limits

- [ ] Rerun the console-interrupt tests (`ProjectSignalTests`,
  `AnalysisSignalTests`, `WindowsConsoleTests`, `TestLifetimeTests`, and the
  `cancel: True` cases) from an ordinary terminal. In the review session they
  also failed on an unchanged baseline because Ctrl+C was not delivered.
- [ ] Rerun AOT and native-signature publishing tests on a machine where
  `vswhere.exe` is on `PATH`; the review machine could not link AOT outputs.
