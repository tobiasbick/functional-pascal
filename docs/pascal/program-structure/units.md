# Units

The unit system enables multi-file projects. Each source file declares its namespace via a `unit` declaration. All project files are listed in the project `.fpasprj` file (see [Projects](projects.md)).

Formal syntax: [`grammar.ebnf`](../../specs/grammar.ebnf) (`unit`, `program`, `uses_clause`).

## Unit declaration

A unit file starts with a `unit` declaration followed by functions, procedures, types, and `const` or `var` bindings. There is no main block.

```pascal
unit MyApp.Utils;
uses Std.Str as Str;

public function Clamp(Value: integer; Min: integer; Max: integer): integer;
begin
  if Value < Min then
    return Min;
  elsif Value > Max then
    return Max;
  else
    return Value;
  end if;
end function;

public function IsBlank(S: string): boolean;
begin
  return Str.Length(Str.Trim(S)) = 0;
end function;
end unit;

```

## Program file

The program file uses a `program` declaration instead of `unit`. It does not define a namespace and is the entry point of the application. There is exactly one program file per project. See [Projects](projects.md) for project structure details.

## Using units

Import each unit with its own `uses Unit.Name as Alias;` declaration. The alias
is the only source access path and opens no short names. Listing a source in a
project manifest does not import it. These rules apply to `Std.*` and source units
alike.

```pascal
program Main;

uses MyApp.Utils as Utils;
uses Std.Console as Console;

begin
  const Clamped: integer := Utils.Clamp(150, 0, 100);
  Console.WriteLn(Clamped);
end program;
```

After `uses Std.Str as Text;`, call `Text.Trim(' value ')`. Neither `Trim(...)`,
`Str.Trim(...)`, nor `Std.Str.Trim(...)` accesses that import. Imported types,
constants, variables, routines, and enum members use the same alias prefix.
An imported type may expose record members using its ordinary member syntax.
Import aliases do not re-export the imported unit's names.

An alias may have the same name as an exported member of its unit. For example,
after `uses Demo.Data as Values;`, `Values.Values` accesses the public member
`Values` in that unit. The alias is a namespace, not an imported storage binding;
field/index paths continue from the selected member.

An alias names exactly its imported unit. For example, `uses Std.Net as Net;`
does not expose the separate unit `Std.Net.Utf8` as `Net.Utf8`. Import it with
`uses Std.Net.Utf8 as Utf8;` and call `Utf8.Encode(...)` instead.

## Alias collisions and visibility

Aliases are case-insensitive and reserved throughout the compilation unit's
lexical scopes. A local declaration, routine parameter, generic parameter, loop
variable, or pattern binding cannot shadow an alias. Different imports cannot
share an alias, and importing the same unit twice is an error even with different
aliases. Choose a distinct alias when a local declaration already uses the name.

Two units may export the same member name: `Text.Length(...)` and
`Arrays.Length(...)` stay distinct under unrelated imports. Bare imported names
are always invalid, including when only one imported unit exports that name.
Lexically declared names keep their normal meaning.

Imports expose public declarations only. An alias does not bypass visibility
checks for types, routines, variables, or record members. Transitive imports are
available for linking and type identity, but must be imported explicitly before
source code can name their declarations. See [Visibility](visibility.md).

## Declarations and type lookup

Each declaration repeats its own keyword, including `public type`, `public const`,
and `public var`. A type and routine cannot share a name in the same scope.
Type headers are collected across the compilation unit before routine signatures
and type bodies are checked. A signature or record field can therefore refer to
a type declared later in the file. Constant and variable initializers, including
record field defaults, retain declaration-order visibility.

Units end with `end unit;` and contain declarations only; programs end with
`end program;` after their required main block.

## Reserved namespace `Std`

The first segment `Std` (ASCII, any case) is reserved for the standard library. User-defined units use another root segment (for example `MyApp.Utils`).

Intrinsic standard-library entries are two-part names such as `Std.Console`. Trusted source standard-library manifests may also export multi-segment units such as `Std.Tools.Format`; only units listed in that manifest's `[exports].units` may appear in an application `uses` clause.

Unknown `uses` entries referring to `Std.*`, and private source standard-library units, are rejected with an error.

## Unit resolution

Units are resolved through the project `.fpasprj` file, which lists all source files belonging to the project. Each file declares its namespace via its `unit` declaration. The directory structure has no influence on the unit name — only the `unit` declaration inside the file matters.

Library units from other projects enter the unit graph when the consuming `.fpasprj` lists them under `[dependencies].projects` (path to another manifest) or `[dependencies].workspace` (member `project.name` inside an enclosing `.fpasworkspace`). A library may restrict which units are importable via `[exports].units` in its `.fpasprj`. See [Projects](projects.md).

Only units reachable from the program file's `uses` chain (including transitive dependencies) are compiled into the final program.

## Compiled-unit sidecars

Project units are compiled independently. Compiling `Geometry.fpas` creates the derived
`Geometry.fpascu` file beside its source. The sidecar contains:

- the unit's public semantic interface;
- relocatable bytecode and source locations;
- source, compiler, bytecode-format, and compilation-option identities;
- direct dependency interface hashes.

`fpas check`, `fpas run`, and `fpas test` automatically rebuild a missing, stale, corrupt, or
incompatible sidecar. Sources and manifests remain authoritative; `.fpascu` files are replaceable
build products and are excluded from source discovery and formatting.

Before reuse, the compiler limits a complete sidecar to 136 MiB and each semantic-interface or
relocatable-object payload to 64 MiB. It verifies payload hashes, requires the envelope,
interface, and object to identify the same unit, rejects duplicate public symbols under Pascal's
ASCII case-insensitive name rules, and validates that the object has exactly one final `Halt`.
Any failure classifies the derived sidecar as corrupt and rebuilds it from the authoritative
source.

A valid unchanged sidecar lets the compilation stage reuse the stored interface and object without
parsing or semantically analyzing the implementation again. Project loading still reads current
source declarations to construct the authoritative Unit graph. A non-public implementation change
rebuilds that unit but does not invalidate consumers while its public interface hash stays
unchanged. An exported signature, layout, constant value, or other public-interface change
invalidates consuming units.

For an exported record, the semantic interface also stores its declaring unit
and the names of non-public record members. Consumers therefore receive the
complete runtime layout needed for linking while semantic analysis still
enforces record member visibility. Before emitting that interface, semantic
analysis rejects any public declaration whose signature or complete exported
type layout refers to a type that is private to the same unit. The diagnostic
identifies the declaration and private type and recommends making the type
public or no longer exporting the declaration.

The final executable bytecode image links only reachable unit objects in dependency order.
Existing top-level constant and variable initializers run in that same dependency order before
the program body. Units do not have separate initialization or finalization syntax.

Before returning that image, the linker requires every callable definition to have a matching
function-table entry in the same object, verifies that Unit function entries remain valid after
the Unit's terminal `Halt` is removed, and applies the complete executable-bytecode validator.
Malformed relocations, callable metadata, startup control flow, or opcode operands therefore fail
during linking rather than being deferred to execution.

Sidecars are staged and validated in the source directory, then replaced in one filesystem
operation. A staging or replacement failure leaves the previous sidecar at its path. Concurrent
readers and writers use a
persistent `.fpascu.lock` coordination file when it exists. The file contains no unit data and is
ignored by Git; operating-system lock ownership is released automatically if the compiler exits,
so a lock is never reclaimed merely because it is old. Reading an existing valid sidecar does not
create a missing lock file, so a read-only source tree remains usable. If rebuilding is required,
the command reports the source-adjacent file it could not publish.

## Implementation (contributors)

Alias registration and source-name qualification live in
`crates/fpas-sema/src/check/name_resolution/imports.rs`. Semantic metadata retains
the mapping to canonical unit identities for compiler lowering and linking.
Source-unit interfaces expose qualified public symbols. Supporting interfaces
supply type identities without opening additional source names.

Regression coverage includes `crates/fpas-sema/tests/syntax_and_names.rs` and
`crates/fpas-cli/src/main_tests/projects/qualified_globals.rs`.

## See also

- [Visibility](visibility.md)
- [Projects](projects.md)
- [Standard library](../std/README.md)
