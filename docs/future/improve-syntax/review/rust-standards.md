# Rust standards review

No active findings. This file is ready for the **Standards** axis of the
[next review](README.md), separate from compiler and tooling spec defects.
Completed coverage is in [reviewed-scope.md](reviewed-scope.md).

## Finding format

Use one heading per independent finding: `Rnn — Pn: concrete violation`.
Each finding must contain:

1. **Status and scope:** fixed reviewed revision and affected crate/module.
2. **Applicable rule:** link the exact requirement from
   [AGENTS.md](../../../../AGENTS.md),
   [development-process.md](../development-process.md) or an applicable skill.
3. **Evidence and impact:** code locations, mixed responsibilities, duplication,
   dead owners, unclear interfaces or test coupling, and their concrete cost.
   File size, speculative performance costs and personal style alone do not
   establish an actionable finding.
4. **Bounded repair:** existing owner to reuse, actual paths to split/move/remove
   and acceptance criteria. Keep the reorganization within the affected concern.
5. **Closure evidence:** required regression checks, second review, docs status
   and commit, linked from [verification.md](verification.md).

Report inspected standards areas even when no independent finding is confirmed.
Do not count a spec defect again solely because it also violates completion
expectations. Refactor-only repairs must preserve behavior and pass existing tests.
