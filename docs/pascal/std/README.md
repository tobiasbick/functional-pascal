# Standard library reference (`Std.*`)

Standard-library units under the reserved `Std` namespace. Import via `uses` and refer to symbols by short or fully qualified names:

```pascal
program Hello;

uses
  Std.Console,
  Std.Math;

begin
  WriteLn(Sqrt(16.0));
  Std.Console.WriteLn(Std.Math.Sqrt(16.0));
end.
```

See [Units](../program-structure/units.md) for `uses` rules and the reserved `Std` namespace.

Source standard-library units are loaded from the `lib/stdlib.fpasprj` manifest beside `fpas` and
use the same source-adjacent `.fpascu` compilation model as project units. Distribution staging
discards staged sidecars, compiles every current unit with the current compiler identity, and
replaces the delivered `lib` tree exactly. Removed units and obsolete build artifacts therefore
cannot remain beside the executable. Commands validate and reuse the delivered sidecars or rebuild
them from source when needed. The units remain implementation-owned:
user projects cannot declare units under `Std.*`. The manifest controls which source units are
public; its private implementation units cannot be imported by applications. Use
`fpas run --std-lib <directory> …`, `fpas check --std-lib <directory> …`, or
`fpas test --std-lib <directory> …` to replace the complete source standard library for that
invocation.

Each unit page is a **self-contained handbook**: importing and short vs qualified names, a **quick reference** table, then routines and types with parameters, behavior, edge cases, and examples.

## Areas

| Area | Hub | Units / topics |
|------|-----|----------------|
| Console | [console/](console/README.md) | Text I/O, retained cells/frames, CRT screen, keyboard, events |
| Host I/O | [host/](host/README.md) | Args, Env, Fs, Path, Proc, Time |
| Text | [text/](text/README.md) | Str, Conv, Parse, Json, Toml |
| Collections | [collections/](collections/README.md) | Array, Dict |
| Numeric | [numeric/](numeric/README.md) | Math, Random |
| Result / Option | [result/](result/README.md) | Result, Option helpers |
| Concurrency | [concurrency/](concurrency/README.md) | Task (`Wait`, `WaitAll`) |
| Terminal UI | [tui/](tui/README.md) | MVU element trees, deterministic headless routing and snapshots |
| Graphics | [graph/](graph/README.md) | Window, drawing, hosted dispatch |
| Testing | [testing/](testing/README.md) | Std.Test assertions |
| Version | [version.md](version.md) | Compiler and library version constants |

## Quick examples

### Console I/O

```pascal
uses Std.Console;

WriteLn('Hello!');
TextColorRGB(255, 160, 0);
WriteLn('Accent text');
NormVideo();
```

Fullscreen code can batch explicit cells with `BeginFrame`, `WriteCells`, and `Present`; see
[Cells and frames](console/cells-frames.md).

### Error handling helpers

```pascal
uses Std.Result, Std.Option;

var R: Result of integer, string := Ok(42);
WriteLn(Std.Result.Unwrap(R));
```

Language rules: [Error handling](../language/error-handling/README.md).

## Shared implementation touchpoints

When changing a `Std.*` API, update docs and:

- Intrinsic opcodes: [`crates/fpas-bytecode/src/intrinsic/mod.rs`](../../../crates/fpas-bytecode/src/intrinsic/mod.rs)
- Intrinsic dispatch: [`crates/fpas-std/src/intrinsics.rs`](../../../crates/fpas-std/src/intrinsics.rs)
- Types and `uses` registration: [`crates/fpas-sema/src/std_registry/`](../../../crates/fpas-sema/src/std_registry/mod.rs)

## See also

- [Units](../program-structure/units.md)
- [Concurrency](../language/concurrency/README.md)
- [Testing](testing/README.md)
