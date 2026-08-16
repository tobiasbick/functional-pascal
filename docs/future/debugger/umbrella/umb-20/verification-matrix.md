# UMB-20 verification matrix

Statuses: `pending`, `pass`, `baseline`, `blocked`, or `rejected`.

| ID | Requirement | Owner test or command | Status | Evidence |
|---|---|---|---|---|
| `U20-V01` | Canonical and short selectors bind exact `FunctionId` values | VM function-breakpoint tests | pass | `routines::canonical_qualified_short...`; focused breakpoint suite passes 7/7 |
| `U20-V02` | Nested and same-named matches bind all exact matches deterministically | VM collision tests | pass | short selector multi-match test binds ordered IDs 1 and 2 |
| `U20-V03` | Missing/no-entry matches remain unverified with a reason | VM and adapter negative tests | pass | VM missing/no-entry plus JSONL/DAP unverified-message tests pass |
| `U20-V04` | Replace-all and limit rejection are atomic | VM breakpoint-state tests | pass | logical, physical-binding, and selector-byte limit tests preserve prior state |
| `U20-V05` | Recursion and multiple tasks retain one logical ID and correct hits | VM execution tests | pass | recursive and task entry test passes with one logical ID |
| `U20-V06` | Source/function collisions yield ordered IDs and one stop | VM ordering tests | pass | shared-boundary VM and JSONL source-log/function-stop tests pass |
| `U20-V07` | JSONL function commands use the shared engine | JSONL integration test | pass | `function_breakpoints` integration suite passes 4/4 |
| `U20-V08` | DAP advertises and implements standard function breakpoints | DAP integration test | pass | `dap_function_breakpoints` passes 3/3 |
| `U20-V09` | VS Code sets and hits an ordinary function breakpoint | extension-host test | pass | final `npm test` passes real FunctionBreakpoint host scenario |
| `U20-V10` | Runtime failures stop by default | shared and adapter failure tests | pass | JSONL default-all and existing DAP inspectable-failure tests pass |
| `U20-V11` | Exact selected diagnostic codes stop | shared and adapter failure tests | pass | exact F4001 JSONL and DAP tests pass |
| `U20-V12` | Nonmatching failures emit diagnostics and terminate unsuccessfully without a stop | shared and adapter failure tests | pass | JSONL and DAP assert diagnostic before exit 1/termination and no stop |
| `U20-V13` | Unknown codes and excessive filter sets reject atomically | shared and adapter negative tests | pass | reserved, duplicate, mixed, and 65-filter replacements preserve default state |
| `U20-V14` | DAP and JSONL expose equivalent filter state and terminal events | adapter parity tests | pass | both adapter suites pass matching and nonmatching outcomes |
| `U20-V15` | Categories are central and exhaustive or not exposed | diagnostic catalog test/design evidence | pass | every central `RUNTIME_ALLOCATED_CODES` entry parses; categories are not exposed |
| `U20-V16` | Conditions and hit counts work on function breakpoints | shared and adapter policy tests | pass | JSONL Boolean condition and JSONL/DAP second-hit tests pass |
| `U20-V17` | Same-boundary logs precede the single stop deterministically | shared and adapter ordering tests | pass | JSONL source log precedes combined ordered-ID function stop |
| `U20-V18` | Policy errors and bounds cannot mutate debuggee state | shared negative tests | pass | invalid policy, limits, logMessage, and mutating action reject before replace |
| `U20-V19` | Current debugger docs match only implemented behavior | docs review plus integration examples | pass | debugger, JSONL, and DAP tool pages updated; language spec unchanged |
| `U20-V20` | Formatting and diff hygiene pass | `cargo fmt --all -- --check`; `git diff --check` | pass | both commands pass; only line-ending notices emitted by Git |
| `U20-V21` | Workspace builds | `cargo build --workspace --locked` | pass | workspace build passes |
| `U20-V22` | Changed Rust libraries are warning-free | targeted `cargo clippy ... -- -D warnings` | pass | diagnostics, VM, and debugger library Clippy passes; unrelated all-target test findings remain outside gate |
| `U20-V23` | Focused VM/debugger tests pass | targeted `cargo test` commands recorded in progress | pass | VM breakpoints 7/7; debug unit 10/10; JSONL/DAP integration 12/12 |
| `U20-V24` | VS Code suite passes | `npm test` in `editors/vscode` | pass | final extension diagnostics/editor/debugger/lifecycle suite passes |
| `U20-V25` | Workspace regression baseline is classified | `cargo test --workspace --locked --no-fail-fast` | pass | complete workspace and doctest run exits 0 |

## Evidence rules

- Replace `—` with the exact command, result, and relevant test count or test
  name.
- Do not mark a row pass from source inspection alone when an executable test
  is feasible.
- Do not mark the package complete while a package-owned row is `pending` or
  `blocked`.
- A `baseline` row must name the independent failing test and show that focused
  UMB-20 tests pass.
