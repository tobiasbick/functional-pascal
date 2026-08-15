# UMB-20 verification matrix

Statuses: `pending`, `pass`, `baseline`, `blocked`, or `rejected`.

| ID | Requirement | Owner test or command | Status | Evidence |
|---|---|---|---|---|
| `U20-V01` | Canonical and short selectors bind exact `FunctionId` values | VM function-breakpoint tests | pending | — |
| `U20-V02` | Nested and same-named matches bind all exact matches deterministically | VM collision tests | pending | — |
| `U20-V03` | Missing/no-entry matches remain unverified with a reason | VM and adapter negative tests | pending | — |
| `U20-V04` | Replace-all and limit rejection are atomic | VM breakpoint-state tests | pending | — |
| `U20-V05` | Recursion and multiple tasks retain one logical ID and correct hits | VM execution tests | pending | — |
| `U20-V06` | Source/function collisions yield ordered IDs and one stop | VM ordering tests | pending | — |
| `U20-V07` | JSONL function commands use the shared engine | JSONL integration test | pending | — |
| `U20-V08` | DAP advertises and implements standard function breakpoints | DAP integration test | pending | — |
| `U20-V09` | VS Code sets and hits an ordinary function breakpoint | extension-host test | pending | — |
| `U20-V10` | Runtime failures stop by default | shared and adapter failure tests | pending | — |
| `U20-V11` | Exact selected diagnostic codes stop | shared and adapter failure tests | pending | — |
| `U20-V12` | Nonmatching failures emit diagnostics and terminate unsuccessfully without a stop | shared and adapter failure tests | pending | — |
| `U20-V13` | Unknown codes and excessive filter sets reject atomically | shared and adapter negative tests | pending | — |
| `U20-V14` | DAP and JSONL expose equivalent filter state and terminal events | adapter parity tests | pending | — |
| `U20-V15` | Categories are central and exhaustive or not exposed | diagnostic catalog test/design evidence | pending | — |
| `U20-V16` | Conditions and hit counts work on function breakpoints | shared and adapter policy tests | pending | — |
| `U20-V17` | Same-boundary logs precede the single stop deterministically | shared and adapter ordering tests | pending | — |
| `U20-V18` | Policy errors and bounds cannot mutate debuggee state | shared negative tests | pending | — |
| `U20-V19` | Current debugger docs match only implemented behavior | docs review plus integration examples | pending | — |
| `U20-V20` | Formatting and diff hygiene pass | `cargo fmt --all -- --check`; `git diff --check` | pending | — |
| `U20-V21` | Workspace builds | `cargo build --workspace --locked` | pending | — |
| `U20-V22` | Changed Rust libraries are warning-free | targeted `cargo clippy ... -- -D warnings` | pending | — |
| `U20-V23` | Focused VM/debugger tests pass | targeted `cargo test` commands recorded in progress | pending | — |
| `U20-V24` | VS Code suite passes | `npm test` in `editors/vscode` | pending | — |
| `U20-V25` | Workspace regression baseline is classified | `cargo test --workspace --locked --no-fail-fast` | pending | — |

## Evidence rules

- Replace `—` with the exact command, result, and relevant test count or test
  name.
- Do not mark a row pass from source inspection alone when an executable test
  is feasible.
- Do not mark the package complete while a package-owned row is `pending` or
  `blocked`.
- A `baseline` row must name the independent failing test and show that focused
  UMB-20 tests pass.
