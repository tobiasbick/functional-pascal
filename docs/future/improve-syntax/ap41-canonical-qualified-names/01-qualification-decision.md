# AP41.1: Qualification decision

Package: [AP41](README.md). Status: complete; agreed contract recorded.
This decision records the approved contract. Implementation and migration are
delivered in [AP41.2](02-qualified-name-rules.md).

## Scope and deliverable

The selected goal is one source form per imported symbol or user enum variant,
with short explicit qualifiers and stable resolution. Use mandatory language
checks rather than an optional authoring/lint convention. Alias-qualified access
retains AP05's existing mechanism and keeps long unit paths in the import list.

## Imports and source names

Every direct import requires `as Alias`, including intrinsic and source Std
units. A file without imported dependencies needs no `uses` clause. Access to
imported public routines, routine values, task targets, types, constants, and
variables uses the source-local alias. A qualifier never imports a dependency.

```pascal
uses Std.Console as Console, MyApp.Domain as Domain;
```

The required forms include `Console.WriteLn(...)`, `Domain.Customer`, and
`Domain.Color.Red`. Plain `uses Std.Console;`, bare imported `WriteLn(...)`,
and original-path access such as `Std.Console.WriteLn(...)` are rejected by
AP41.2. Aliases expose neither sibling units nor private/transitive exports.

Preserve AP05's identity and collision rules:

- Import each canonical unit at most once per source file, case-insensitively.
  Shared transitive dependencies remain valid.
- Aliases must be distinct from other aliases, reserved type names, imported or
  declaring unit namespace roots, and declarations in the importing file.
  Parameters, generic parameters, locals, loop variables, and pattern bindings
  cannot shadow an alias. Diagnose the conflict and show an alias/declaration repair.
- Aliases remain local to a file and are not re-exported. Exported type identities,
  interfaces, unit dependencies, and linked symbols use the canonical unit name.
  Different importing files may give the same unit different explicit aliases.
- `as` remains contextual import syntax. Normal names remain case-insensitive.

## User enums and built-in variants

Use `Type.Variant` for locally visible user enum types, including explicit local
type aliases, and `Alias.Type.Variant` when the type is accessed through an import.
Remove short variant lookup even when the selector or expected type would make
it unambiguous. Imported `Alias.Variant` does not substitute for the enum type.

Apply the rule to payload-free values, payload constructors, comparisons, case
labels, nested patterns, and Boolean `is` tests. Keep generic type applications,
inference, and explicit type aliases under AP24/AP34. AP36's positional-or-named
construction/pattern fields remain valid, with an identical qualification rule.

`Some`, `None`, `Ok`, and `Error` remain reserved Option/Result forms in expressions
and patterns. Keep their existing type inference, payload labels, and argument
styles; introduce no `Option.Some`/`Result.Ok` alternatives or import requirement.

Ordinary local routines and variables retain their names and lexical shadowing
rules. Record members and import-independent native operations such as
`Items.Length()` retain their existing dot syntax. This decision removes no
AP06 operation and introduces no free-function receiver fallback.

## Editor behavior and migration

Auto-import and completion insert a required alias qualifier together with any
new aliased import. Reuse an existing canonical direct import and its written
alias rather than adding a duplicate or renaming it. The default new alias is
the final unit-name segment. When reserved or already used, select a deterministic,
readable nonconflicting alias derived from the unit path under the same compiler
reservations; alias spelling remains a local naming choice rather than a second
language form. For example, colliding `App.Model` and `Core.Model` imports can use
`Model` and `CoreModel`. Adding an import never rewrites existing aliases.

Completion and rename preserve canonical symbol identity and required enum type
qualifiers. Diagnostics for a missing alias, unqualified imported name, or short
enum variant show the applicable canonical repair. Migrate by resolved symbols,
including imported types in public signatures and pattern targets; matching text
alone cannot distinguish a local routine from an imported routine or enum member.

## Decision example matrix

These are the delivered AP41.2 outcomes. Assume the
named declarations are public and valid under their ordinary type/lifetime rules.

| Situation | Selected outcome |
|-----------|------------------|
| `uses Std.Console as Console;` followed by `Console.WriteLn(...)` | Accept |
| Plain `uses Std.Console;` | Reject; require `as Console` or another valid explicit alias |
| Aliased Console import followed by `WriteLn(...)` | Reject as an imported short name; show `Console.WriteLn(...)` |
| Aliased Console import followed by `Std.Console.WriteLn(...)` | Reject; use the declared alias |
| A separately declared local `WriteLn` routine | Keep ordinary local name resolution; no imported short name competes |
| `Domain.Customer` in a binding or exported signature | Accept; preserve the canonical public type identity |
| Local enum value or pattern `Color.Red` | Accept |
| Bare local enum variant `Red`, even with fixed `Color` context | Reject; show `Color.Red` |
| Imported enum value or pattern `Domain.Color.Red` | Accept |
| Imported short variant `Domain.Red` | Reject; show the enum type qualifier |
| Another enum also declares `Red` | Qualified existing targets remain unchanged |
| Enum `Shape` declares `Circle`, and a separate type is named `Circle` | `Shape.Circle` remains valid; no short variant lookup competes |
| Generic user constructor `Domain.Lookup.Found(42)` | Accept when AP24/AP34 determine every type argument |
| `Some(Color.Red)`, `None`, `Ok(...)`, or `Error(...)` | Retain the existing reserved forms and typing rules |
| `Option.Some(...)` or `Result.Ok(...)` as alternate built-in forms | Reject; retain the reserved spelling |
| Repeated import, duplicate/reserved alias, or local alias shadowing | Reject under retained AP05 collision rules |
| Auto-importing two units with the same final namespace segment | Choose distinct valid aliases and insert each qualifier |
| Renaming a file-local alias | Change its declaration and uses; retain canonical dependency identity |
| Native `Items.Length()` or an ordinary record method | Keep existing member lookup and import requirements |

The following complete example uses the selected forms already supported by the
current compiler; AP41.2 also rejects the alternative forms:

```pascal
program QualifiedNames;

uses Std.Console as Console;

type Color = enum
  Red;
  Green;
end enum;

begin
  const Selected := Color.Red;
  case Selected of
    when Color.Red:
      Console.WriteLn('red');
    when Color.Green:
      Console.WriteLn('green');
  end case;
end.
```

## Affected areas and documentation

The decision is implemented and documented in
[units](../../../pascal/program-structure/units.md) and
[enums](../../../pascal/language/types/enums.md), the grammar, diagnostics,
compact reference, editor guidance, and authoring skill.

AP41.2 explicitly replaces AP05's plain-import allowance and optional short
user-enum variant names while preserving the completed packages' status and
their remaining construction, binding, inference, and editor contracts.

## Open decisions and verification

None for the selected contract. The matrix fixes compiler enforcement, accepted
and rejected forms, collision behavior, built-in variants, and editor output.
AP41.2 supplies language-boundary, compiled-unit, migration, and tooling
regressions; implementation and independent review are complete.
