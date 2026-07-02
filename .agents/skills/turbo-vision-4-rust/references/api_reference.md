# Examples

Use these examples to calibrate behavior when this skill triggers.

## Example 1: Add a new `Std.Tui` call

User request:

```text
add Application.Foo for Turbo Vision
```

Expected agent behavior:

- Read `docs/pascal/std/tui/app/vm-bridge.md` end-to-end recipe.
- Follow `.agents/skills/fpas-change-checklist/SKILL.md`.
- Verify upstream `turbo-vision` API from source or crates.io when bridging new widgets.
- Add sema, compiler, bytecode, VM, docs, and tests in one focused change.

## Example 2: Fix live TUI behavior

User request:

```text
checkbox mouse click does not work in Application.Run
```

Expected agent behavior:

- Read `docs/pascal/std/tui/app/controls.md` and `vm-bridge.md`.
- Inspect `crates/fpas-vm/src/vm/execute/io/tui/` bridge modules.
- Check whether upstream `turbo-vision` lacks the behavior before patching the FPAS facade.
- Add Rust and/or FPAS regression tests under `tests/tui/controls/`.

## Example 3: User asks about upstream API details

User request:

```text
what Button constructor does turbo-vision currently expose?
```

Expected agent behavior:

- Browse or fetch upstream source instead of relying on memory.
- Cite the upstream file or summarize the exact checked signature.
- If network access is unavailable, say the answer is unverified and may be stale.
