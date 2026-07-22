# Legacy TUI retirement inventory

This Phase 6.1 inventory identifies retired TUI material for deletion. The
current Tui3 implementation does not support the old IDE source, which is
explicitly excluded from builds and tests. `Std.Tui2` has no application
consumer.

## Reproduce the findings

Run these commands from the repository root.  `bin/` and `target/` are generated
copies and build output, not migration sources.

```powershell
rg -n -i '^uses .*Std\.Tui(\.|;|,)' apps examples tests lib
rg -n -i '^uses .*Std\.Tui2(\.|;|,)' apps examples tests
rg -l -i -g '!target/**' -g '!bin/**' 'Std\.Tui|Turbo.?Vision|turbo_vision|turbo-vision|io/tui|io\\tui' apps docs crates Cargo.toml AGENTS.md AI_CONTRIBUTING.md .agents
rg --files lib/Std/Tui lib/Std/Tui2
```

The first query identifies the IDE's direct `Std.Tui` imports.  The second is
expected to have no match under `apps/` or `examples/`; its matches are the
dedicated `tests/stdlib/tui2/` regression suite.

## Classified paths

| Area | Paths covered | Classification | Reason |
| --- | --- | --- | --- |
| Current IDE facade | `lib/Std/Tui.fpas`, `lib/Std/Tui/**`; Tui compiler, bytecode, sema, std, VM bridge, Cargo and guidance files returned by the third query | Delete | Retired implementation; Tui3 does not provide compatibility. |
| IDE application | `apps/ide/src/**`, `apps/ide/*.fpasprj`, `apps/ide/ide.fpasworkspace` | Intentionally retain | Legacy source only: excluded from Tui3 builds and tests, then removed with the retired stack. |
| Legacy `Std.Tui` examples and tests | `examples/pascal/tui/**`, `tests/tui/**`, related current-Tui documentation | Delete | They solely exercise the facade replaced after promotion; do not port them before the IDE flow proves the required behavior. |
| Abandoned Tui2 implementation | `lib/Std/Tui2.fpas`, `lib/Std/Tui2/**`, `tests/stdlib/tui2/**`, `docs/pascal/std/tui2/**`, `docs/future/tui2/**` | Delete | Tui2 has no application or example consumer.  Keep it only until the approved Phase 7 deletion. |
| Temporary Tui3 implementation | `lib/Std/Tui3.fpas`, `lib/Std/Tui3/**`, `tests/stdlib/tui3/**`, `examples/pascal/tui3/**`, `docs/pascal/std/tui3/**`, `docs/future/tui3/**` | Retain | This is the implementation target and its regression coverage.  Its public paths are renamed at promotion; its completed planning material is then deleted. |

## Legacy IDE boundary

`apps/ide/README.md` records the source as retired. Its dedicated test project
and test sources are deleted, and it is not a Tui3 migration target. No Tui3
test may import an `Ide.*` unit.
