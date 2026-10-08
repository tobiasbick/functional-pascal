# AP06: Native catalog coverage

Package: [AP06: Fixed dot-call targets](README.md)
Catalog: [Built-in type operations](catalog.md)

Status: complete.

## Operation coverage

| Receiver or factory owner | Entries |
| --- | ---: |
| String instance | 31 |
| Array instance | 19 |
| Array of string specialization (`Join`) | 1 |
| Dictionary instance | 11 |
| Option instance | 7 |
| Result instance | 7 |
| `string.Chr` factory | 1 |
| `array.Fill` factory | 1 |
| Total | 78 |

The catalog contains 75 distinct operation implementations and three `IsEmpty`
entries.
String `Slice` is the single public substring-range name. Private implementation
IDs identify lowering targets; they do not register public free routines.

## Consumer and API coverage

Native forms cover library, app, example and test sources; embedded and assembled
Rust programs; generators; formatter fixtures; editor declarations; and handbook
examples. Own free functions are ordinarily called, while real record members
retain chaining. Native callback references use wrappers.

The five retired helper units are absent from imports, exported symbols,
editor stubs and auto-imports. Standard-library discovery cannot restore them.
Other Std units still require imports.

## Verification ownership

- `fpas-sema/src/std_registry/native/tests.rs` and
  `fpas-sema/src/std_registry/native/tests/handbook.rs` check
  catalog completeness, naming, collisions, roles and handbook signatures.
- `fpas-sema/src/tests/expr/native.rs` checks every positional/named signature
  without imports, generics, factories and rejection paths.
- Compiler and FPAS regressions check evaluation, named arguments, early try,
  tasks, shared mutation storage and type-changing chains.
- Language-service tests check native completion/signatures and retired APIs.

Paths above are relative to `crates/`. The
[AP06.3 result](03-fixed-dot-resolution.md#result) describes implemented behavior.

## Known analysis boundaries

`tests/stdlib/tui/cell_grid_row_equivalence.fpas` imports the internal unit
`Std.Tui.Rendering.Canvas`, which is not exported by `lib/stdlib.fpasprj`.
That independent project boundary does not change native-operation availability;
its type-operation calls use native forms. Formatter inputs with fictional user
units and deliberate negative programs are syntax/test fixtures rather than
standalone semantic programs.
