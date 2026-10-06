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
discards staged sidecars, compiles every unit with the compiler identity, and replaces the
delivered `lib` tree exactly. The delivered tree therefore contains only manifest sources and their
matching build artifacts. Commands validate and reuse the delivered sidecars or rebuild
them from source when needed. The units remain implementation-owned:
user projects cannot declare units under `Std.*`. The manifest controls which source units are
public; its private implementation units cannot be imported by applications. Use
`fpas run --std-lib <directory> …`, `fpas check --std-lib <directory> …`, or
`fpas test --std-lib <directory> …` to replace the complete source standard library for that
invocation.

Each unit page is a **self-contained handbook**: importing and short vs qualified
names, a **quick reference** table, then routines and types with parameters,
behavior, edge cases, and examples. Link implementation locations under
`## Implementation (contributors)` and related pages under `## See also`.

Intrinsic units implemented by the compiler, VM, or Rust runtime also expose
generated editor declarations under [`lib/api/Std/`](../../../lib/api/Std/).
They use the same `//` Markdown documentation as ordinary FPAS source and give
the language server concrete targets for hover, completion, signature help,
**Go to Definition**, and **Go to Type Definition**. These declarations are
not compiled and do not implement runtime behavior. Their maintenance is covered
under [Shared implementation touchpoints](#shared-implementation-touchpoints).

## Areas

| Area | Hub | Units / topics |
|------|-----|----------------|
| Console | [console/](console/README.md) | Text I/O, retained cells/frames, CRT screen, keyboard, events |
| Host I/O | [host/](host/README.md) | Args, Env, Fs, Path, Proc, Time |
| Networking | [network/](network/README.md) | Net, Server lifetime, URI, UTF-8, HTTP |
| AI clients | [ai/](ai/README.md) | OpenAI-compatible chat completions |
| Text | [text/](text/README.md) | Str, Conv, Parse, Json, Json.Fields, Toml, Toml.Fields |
| Collections | [collections/](collections/README.md) | Array, Dict |
| Numeric | [numeric/](numeric/README.md) | Math, Bits, Random |
| Cryptography | [cryptography/](cryptography/README.md) | Operating-system random bytes and secure integers |
| Result / Option | [result/](result/README.md) | Result, Option helpers |
| Concurrency | [concurrency/](concurrency/README.md) | Task (bounded channels, cooperative cancellation, task waits, typed `Select` cases, task groups, supervision) |
| Terminal UI | [tui/](tui/README.md) | MVU element trees, deterministic headless routing and snapshots |
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
uses Std.Results, Std.Options;

var R: Result of integer, string := Ok(42);
WriteLn(Std.Results.Unwrap(R));
```

Language rules: [Error handling](../language/error-handling/README.md).

## Shared implementation touchpoints

When changing a `Std.*` API, update its handbook and every applicable layer:

| Concern | Location |
|---------|----------|
| Intrinsic unit and symbol names | [`fpas-std/src/std_units/`](../../../crates/fpas-std/src/std_units/mod.rs) |
| Types, signatures, and `uses` registration | [`fpas-sema/src/std_registry/`](../../../crates/fpas-sema/src/std_registry/mod.rs) |
| Compiler catalog and lowering | [`intrinsic_catalog.rs`](../../../crates/fpas-compiler/src/intrinsic_catalog.rs), [`lowering/calls.rs`](../../../crates/fpas-compiler/src/lowering/calls.rs), [`selection/intrinsics.rs`](../../../crates/fpas-compiler/src/bytecode/selection/intrinsics.rs) |
| Intrinsic IDs and metadata | [`fpas-bytecode/src/intrinsic/`](../../../crates/fpas-bytecode/src/intrinsic/mod.rs) |
| Runtime dispatch and execution | [`fpas-std/src/intrinsics.rs`](../../../crates/fpas-std/src/intrinsics.rs), [`fpas-vm/src/vm/`](../../../crates/fpas-vm/src/vm/) |
| Source standard-library API | [`lib/Std/`](../../../lib/Std/), [`lib/stdlib.fpasprj`](../../../lib/stdlib.fpasprj) |
| Regression coverage | Owning crate tests and [`tests/stdlib/`](../../../tests/stdlib/) or the relevant runner, console, concurrency, or app tests |

Source units such as `Std.Tui` expose their API directly in `lib/Std/`.
Console, network, and test hosts live under
[`fpas-vm/src/vm/hosted/`](../../../crates/fpas-vm/src/vm/hosted/).

Regenerate the intrinsic editor declarations under
[`lib/api/Std/`](../../../lib/api/Std/) after an intrinsic API or handbook change:

```text
cargo run -p fpas-sema --example export_intrinsic_std_api
```

## See also

- [Units](../program-structure/units.md)
- [Concurrency](../language/concurrency/README.md)
- [Testing](testing/README.md)
