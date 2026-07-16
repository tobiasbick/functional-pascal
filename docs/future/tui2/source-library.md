# Std.Tui2 source-library packaging

Std.Tui2 is distributed as trusted FPAS source. A manifest, not directory scanning alone, defines the bundled source standard library.

## Distribution layout

```text
lib/
  stdlib.fpasprj
  Std/
    Version.fpas
    Tui2.fpas
    Tui2/
      Geometry.fpas
      Registry.fpas
      Layout/
      Views/
      Controls/
```

`stdlib.fpasprj` is a `kind = "library"` project. Its source list includes `Std/**/*.fpas`, and its exports initially contain `Std.Version` and `Std.Tui2`.

Internal implementation units use names such as `Std.Tui2.Geometry`. Trusted standard-library sources may declare multi-segment `Std.*` names. User projects may not declare any unit whose first segment is `Std`.

## Export boundary

The manifest is authoritative:

- exported units may appear in an application `uses` clause;
- non-exported units are linkable only from other units in the same source standard library;
- a user import of `Std.Tui2.Geometry` fails with an export diagnostic;
- source units do not need an entry in the Rust intrinsic-unit registry;
- intrinsic units such as `Std.Console` continue to use their existing registry.

If a source unit and intrinsic unit declare the same canonical unit name, loading fails. Source code cannot replace individual intrinsic units implicitly.

## Discovery and overrides

The default manifest is `lib/stdlib.fpasprj` beside `fpas`. `--std-lib <directory>` selects `<directory>/stdlib.fpasprj` and replaces the complete source standard library for that invocation.

An explicit override is implementation-trusted and may define `Std.*`, but it must pass the same manifest, export, duplicate-name, and parse validation as the bundled library.

`run`, `check`, and `test` use the selected manifest. `fmt` formats only explicit user paths and does not load the source standard library.

## Validation

Loading rejects:

- a missing manifest;
- a non-library manifest;
- program files in its source list;
- exported names without matching unit declarations;
- duplicate unit names;
- source units outside the `Std` root;
- collisions with intrinsic standard units;
- imports of non-exported implementation units from user code.

This manifest work is Phase 0 and precedes Std.Tui2 implementation.
