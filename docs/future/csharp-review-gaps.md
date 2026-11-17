# C# review verification

Pending platform acceptance. Commands and prerequisites belong to the
[development workflow](../build/development.md) and [test coverage](../../tests/coverage.md).

- [ ] Run console-interrupt coverage from ordinary native terminals:
  `ProjectSignalTests`, `AnalysisSignalTests`, `WindowsConsoleTests`,
  `TestLifetimeTests`, and the `cancel: True` cases.
- [ ] Run Windows-specific debugger/window interaction and native publishing
  coverage, including AOT and delivery signatures, with the native toolchain
  and `vswhere.exe` available.
