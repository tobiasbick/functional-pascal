# Decision Record

## Decision

Rewrite `Std.Tui` around the upstream Rust crate `turbo-vision` instead of continuing to simulate Turbo Vision concepts in the Functional Pascal runtime.

The FPAS API should be native to Pascal-style code. It should not copy the Rust API one-to-one.

## Upstream Facts

- Repository: `https://github.com/aovestdipaperino/turbo-vision-4-rust`
- Crate: `turbo-vision`
- Rust library name: `turbo_vision`
- Checked version: `1.3.1`
- License: MIT
- Rust edition: 2024
- Terminal backend dependency: `crossterm = "0.29"`
- Main implemented areas: application, desktop, windows, dialogs, buttons, input lines, list boxes, menus, status line, scrollbars, memo/editor, file dialog, modal flow, mouse, drag/resize, double buffering, screen dump support.

## Local Facts

- The workspace already uses Rust edition 2024.
- The workspace already has `crossterm = "0.29"` in root `Cargo.toml`.
- MIT is compatible with the workspace BSD-3-Clause license.
- The current `Std.Tui` implementation is spread across sema registration, compiler lowering, bytecode intrinsics, VM host code, `fpas-std` runtime code, docs, examples, tests, and `apps/ide`.

## Goals

- Make `Std.Tui` a small FPAS-facing facade over proven Turbo Vision primitives.
- Prefer Turbo Vision's application, desktop, window, dialog, command, event, and widget model.
- Remove the custom retained TUI engine once the new spine works.
- Keep the public API simple for Pascal authors: handles, records, constants, and callbacks.
- Keep tests authorable from FPAS where possible.

## Non-Goals

- No backward compatibility with current `Application.Host*` APIs.
- No one-to-one Rust API mirror.
- No FPAS exposure of Rust traits, `Box<dyn View>`, builders, ownership details, or module layout.
- No broad compatibility adapter over the old retained view tree.
- No current-spec docs in `docs/pascal/` until behavior is implemented.

## Design Principle

Copy the Turbo Vision architecture, not the Rust surface syntax.

FPAS should feel like Turbo Vision was designed for Pascal:

- `Application`, `Desktop`, `Window`, `Dialog`, `Button`, `InputLine`, `MenuBar`, `StatusLine`.
- `Point`, `Size`, `Rect` as value records.
- Host-owned handles for live UI objects.
- Commands and events as explicit FPAS values.
- Callbacks registered through clear `Application` routines.

## Main Risk

The hard part is callback ownership and VM re-entry, not drawing. A Turbo Vision event must be able to trigger an FPAS callback without violating VM state, borrow rules, or terminal lifecycle invariants.
