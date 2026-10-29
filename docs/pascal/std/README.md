# Standard library reference (`Std.*`)

Standard-library units under the reserved `Std` namespace. Built-in string,
array, dictionary, Option, and Result operations are described in the
[type reference](../language/types/README.md) and are always available through
[dot calls](../language/functions/fluent-calls.md). Import every standard unit with
an explicit alias and access its public symbols through that alias:

```pascal id=example-1
program Hello;

uses Std.Console as Console, Std.Math as Math;

begin
  Console.WriteLn(Math.Sqrt(16.0));
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

Each unit page is a **self-contained handbook**: importing and alias-qualified
names, a **quick reference** table, then routines and types with parameters,
behavior, edge cases, and examples. Link implementation locations under
`## Implementation (contributors)` and related pages under `## See also`.

Intrinsic units implemented by the compiler, VM, or Rust runtime also expose
generated editor declarations under [`lib/api/Std/`](../../../lib/api/Std).
They use the same `//` Markdown documentation as ordinary FPAS source and give
the language server concrete targets for hover, completion, signature help,
**Go to Definition**, and **Go to Type Definition**. These declarations are
not compiled and do not implement runtime behavior. Their maintenance is covered
under [Shared implementation touchpoints](#shared-implementation-touchpoints).

The same declarations and comments answer
[`fpas doc`](../program-structure/cli.md#looking-up-declarations). The comment before each
unit header ends with a `Handbook:` line naming the unit's handbook page, such as
`docs/pascal/std/host/fs.md`.

## Areas

| Area | Hub | Units / topics |
|------|-----|----------------|
| Console | [console/](console/README.md) | Text I/O, retained cells/frames, CRT screen, keyboard, events |
| Host I/O | [host/](host/README.md) | Args, Calendar (Gregorian dates and UTC), Cli (schema parsing/help), Env, Errors, Fs (whole-file and handle I/O), Path, Proc (options and bounded capture), Time |
| Networking | [network/](network/README.md) | Sockets, Server lifetime, URI, HTTP |
| AI protocols | [ai/](ai/README.md) | OpenAI-compatible chat completions, native ACP clients, typed events, host adapters and MCP tool servers |
| Text and formats | [text/](text/README.md) | Text.Format, Text.Parse, Text.Utf8, Encoding (Base64/Base64url/hex), Encoding.Json, Encoding.Toml and field helpers |
| Numeric | [numeric/](numeric/README.md) | Math, Bits, Random |
| Cryptography | [cryptography/](cryptography/README.md) | Operating-system random bytes and secure integers |
| Concurrency | [concurrency/](concurrency/README.md) | Tasks (bounded channels, cooperative cancellation, task waits, typed `Select` cases, task groups, supervision) |
| Terminal UI | [tui/](tui/README.md) | MVU element trees, deterministic headless routing and snapshots |
| Testing | [testing/](testing/README.md) | Std.Test assertions |
| Version | [version.md](version.md) | Compiler and library version constants |

## Quick examples

### Console I/O

```pascal id=example-2 fragment context=header
uses Std.Console as Console;
begin
Console.WriteLn('Hello!');
Console.TextColorRGB(255, 160, 0);
Console.WriteLn('Accent text');
Console.NormVideo();

end.
```

Fullscreen code can batch explicit cells with `BeginFrame`, `WriteCells`, and `Present`; see
[Cells and frames](console/cells-frames.md).

### Error handling helpers

```pascal id=example-3 fragment context=body-console
const R: result of (integer, string) := Ok(42);
Console.WriteLn(R.Unwrap());
```

Language rules: [Error handling](../language/error-handling/README.md).

## Shared implementation touchpoints

Before adding, moving, splitting or renaming a public standard unit, follow the
binding [namespace specification](../../specs/std-namespaces.md). It defines
canonical allocations and extension rules; this handbook documents the available
imports. The [roadmap](../../future/std-roadmap.md) owns future capability
priorities. Names in the specification do not make new modules available.

When changing a `Std.*` API, update its handbook and every applicable layer:

| Concern | Location |
|---------|----------|
| Intrinsic unit and symbol names | [`fpas-builtins/src/std_units/`](../../../crates/fpas-builtins/src/std_units/mod.rs) |
| Types, signatures, and `uses` registration | [`fpas-sema/src/std_registry/`](../../../crates/fpas-sema/src/std_registry/mod.rs) |
| Compiler catalog and lowering | [`intrinsic_catalog.rs`](../../../crates/fpas-compiler/src/intrinsic_catalog.rs), [`lowering/calls.rs`](../../../crates/fpas-compiler/src/lowering/calls.rs), [`selection/intrinsics.rs`](../../../crates/fpas-compiler/src/bytecode/selection/intrinsics.rs) |
| Intrinsic IDs and metadata | [`fpas-code/src/intrinsic/`](../../../crates/fpas-code/src/intrinsic/mod.rs) |
| Numeric arithmetic and text rules | [`fpas-numeric/src/`](../../../crates/fpas-numeric/src/lib.rs), shared by checking, folding, execution, and Std numeric operations |
| Runtime dispatch and execution | [`fpas-runtime/src/stdlib/intrinsics.rs`](../../../crates/fpas-runtime/src/stdlib/intrinsics.rs), [`fpas-runtime/src/execution/vm/`](../../../crates/fpas-runtime/src/execution/vm) |
| Source standard-library API | [`lib/Std/`](../../../lib/Std), [`lib/stdlib.fpasprj`](../../../lib/stdlib.fpasprj) |
| Regression coverage | Owning crate tests and [`tests/stdlib/`](../../../tests/stdlib) or the relevant runner, console, concurrency, or app tests |

Source units such as `Std.Ui.Terminal` expose their API directly in `lib/Std/`. A public source
unit with its own handbook page names it in the comment before its `unit` header, in the same
`Handbook:` form as the generated declarations.
Console, network, and test hosts live under
[`fpas-runtime/src/execution/vm/hosted/`](../../../crates/fpas-runtime/src/execution/vm/hosted).

Regenerate the intrinsic editor declarations under
[`lib/api/Std/`](../../../lib/api/Std) after an intrinsic API or handbook change. The unit comment comes from
the first sentence of the handbook page titled with the unit name. Native built-in signatures and editor
documentation come directly from `fpas-sema/src/std_registry/native/`; the exporter
removes stale declarations of the five former type-helper units:

```text
cargo run -p fpas-sema --example export_intrinsic_std_api
```

## See also

- [Units](../program-structure/units.md)
- [Concurrency](../language/concurrency/README.md)
- [Testing](testing/README.md)
