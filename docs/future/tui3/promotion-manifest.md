# Tui promotion manifest

This is the exact Phase 7.1 manifest. It authorizes no change by itself. The
only retained old-TUI references are the explicitly non-buildable legacy IDE
sources under `apps/ide/`.

## Delete

| Target | Scope |
| --- | --- |
| `lib/Std/Tui2.fpas` and `lib/Std/Tui2/` | Entire abandoned Tui2 public facade and implementation tree. |
| `tests/stdlib/tui2/` | Entire Tui2 FPAS regression suite. |
| `tests/tui/` | Entire Turbo Vision FPAS regression suite. |
| `examples/pascal/tui/` | Entire retired Turbo Vision example tree. |
| `crates/fpas-compiler/src/compiler/std_calls/tui/` | All old-Tui lowering modules. |
| `crates/fpas-sema/src/std_registry/loaded/tui/` | All old-Tui semantic registration modules. |
| `crates/fpas-bytecode/src/intrinsic/tui/` | Old-Tui intrinsic definition module. |
| `crates/fpas-std/src/tui/` | Turbo Vision runtime support and tests. |
| `crates/fpas-vm/src/vm/execute/io/tui/` | Entire Turbo Vision VM bridge and adapters. |
| `crates/fpas-vm/src/vm/shared/tui.rs` | Old-Tui shared session state. |
| `crates/fpas-vm/src/vm/turbo_vision_bool_cell.rs`, `turbo_vision_input_text_cell.rs`, `turbo_vision_list_selection_cell.rs` | Turbo Vision read-back cells. |
| `crates/fpas-vm/src/tests/core/tui_turbo_vision_vm.rs` | Bridge-only VM integration test. |
| `crates/fpas-compiler/src/tests/std_library/tui.rs`, `crates/fpas-sema/src/tests/integration/std_units/tui.rs` | Old-Tui compiler and semantic tests. |
| `docs/pascal/std/tui/`, `docs/pascal/std/tui2/` | Retired user references. |
| `docs/future/tui2/`, `docs/future/tui-bridged-readback.md` | Retired plans and upstream handoff. |
| `docs/future/tui3/` | Completed temporary plan after the rename and checks. |

## Modify

| File | Required change |
| --- | --- |
| `Cargo.toml`, `Cargo.lock`, `crates/fpas-{std,vm}/Cargo.toml` | Remove Turbo Vision dependencies and lock entries. |
| `crates/fpas-std/build.rs` | Remove Turbo Vision command-constant generation. |
| `crates/fpas-std/src/{intrinsics.rs,std_units/{mod.rs,units.rs,symbols/{mod.rs,std_symbols/{mod.rs,tui.rs}}}}` | Remove old-Tui registry and intrinsic wiring. |
| `crates/fpas-sema/src/std_registry/{builtins/tui.rs,loaded/mod.rs}` | Remove old-Tui registration dispatch. |
| `crates/fpas-compiler/src/compiler/{designator/builtin_consts.rs,locals.rs,program/mod.rs,std_aliases.rs,std_calls/mod.rs}` | Remove old-Tui callback, constant, enum, and lowering dispatch. |
| `crates/fpas-bytecode/{Cargo.toml,build.rs,src/{lib.rs,intrinsic/{mod.rs,tests.rs}}}` | Remove old-Tui intrinsic generation, exports, module wiring, and completeness entries. |
| `crates/fpas-vm/src/vm/{mod.rs,shared.rs,execute/mod.rs,execute/io/mod.rs,worker.rs}` | Remove bridge state and execution wiring. |
| `crates/fpas-cli/src/main_tests/{test_runner.rs,test_suite.rs}`, `crates/fpas-fmt/src/emit/decl/mod.rs` | Remove retired-Tui test fixtures. |
| `docs/pascal/{std/README.md,std/console/README.md,std/testing/test.md,apps/README.md,apps/ide.md}`, `examples/README.md` | Point only to promoted Tui; retain the IDE legacy warning. |
| `docs/future/README.md`, `AGENTS.md`, `AI_CONTRIBUTING.md`, `.agents/skills/{fpas-authoring,fpas-change-checklist,turbo-vision-4-rust}/**` | Remove retired bridge/Tui2/Tui3 planning guidance. |
| `lib/stdlib.fpasprj`, `tests/suite.fpasprj` | Remove Tui2 and retired-Tui source/test entries; add promoted Tui paths where needed. |

## Rename

| Current target | Final target |
| --- | --- |
| `lib/Std/Tui3.fpas` | `lib/Std/Tui.fpas` |
| `lib/Std/Tui3/` | `lib/Std/Tui/` |
| `tests/stdlib/tui3/` | `tests/stdlib/tui/` |
| `examples/pascal/tui3/` | `examples/pascal/tui/` |
| `docs/pascal/std/tui3/` | `docs/pascal/std/tui/` |

The renamed source and tests must replace `Std.Tui3` imports and unit names with
`Std.Tui` in the same change. No compatibility alias is retained.

## Retain outside the promoted product

| Target | Reason |
| --- | --- |
| `apps/ide/` | Explicit non-buildable legacy source. `apps/ide/README.md` is the warning; it is excluded from all migration checks. |

## Reference checks

Before deletion, the first query must find the listed retired targets. After
promotion, it must find no result outside `apps/ide/` and its legacy warning.

```powershell
$old = 'Std\.Tui2|Std\.Tui3|Turbo.?Vision|turbo_vision|turbo-vision|io/tui|io\\tui'
rg -n -i -g '!target/**' -g '!bin/**' $old .
rg -n -i -g '!apps/ide/**' -g '!target/**' -g '!bin/**' $old .
rg -n -i -g '!apps/ide/**' -g '!target/**' -g '!bin/**' 'Std\.Tui' .
```

The final query is expected to find the promoted `Std.Tui` implementation,
tests, example, and documentation. The second query is expected to find no
retired names or Turbo Vision references.
