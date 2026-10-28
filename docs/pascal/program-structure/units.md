# Units

The unit system enables multi-file projects. Each source file declares its namespace via a `unit` declaration. All project files are listed in the project `.fpasprj` file (see [Projects](projects.md)).

Formal syntax: [`grammar.ebnf`](../../specs/grammar.ebnf) (`unit`, `program`, `uses_clause`).

## Unit declaration

A unit file starts with a `unit` declaration followed by declarations (functions, procedures, types, constants and variables). Every unit ends with `end unit;` after its last declaration, including a unit with no declarations. There is no main block.

Every type, constant, and variable has its own
`type`, `const`, or `var` keyword. Repeat `public`
before each exported declaration; it applies to that declaration only.

```pascal id=example-1 project=units file=utils.fpas
unit MyApp.Utils;

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
  return S.Trim().Length() = 0;
end function;
end unit;
```

## Program file

The program file uses a `program` declaration instead of `unit`. It does not define a namespace and is the entry point of the application. There is exactly one program file per project. See [Projects](projects.md) for project structure details.

## Using units

Every directly referenced unit requires an explicit aliased import, including
`Std.*` units: `uses Std.Math as Math;`. Imported public names use that alias,
for example `Math.Abs(-3)`. A project source list and a qualified expression
never import a unit automatically.

```pascal id=example-2 project=units file=main.fpas
program Main;

uses MyApp.Utils as Utils, Std.Console as Console;

begin
  const Clamped: integer := Utils.Clamp(150, 0, 100);
  Console.WriteLn(Clamped); // 100
end.
```

## Short names and qualified names

Ordinary local declarations retain their short names. Imported symbols require
`Alias.Name` in expressions, calls, routine values, task targets, type annotations,
and public signatures. User enum variants require their type qualifier, such as
`Domain.Color.Red`; see [Enums](../language/types/enums.md).

```pascal id=example-3
program Hello;

uses Std.Console as Console;

begin
  Console.WriteLn('Hello');
end.
```

### Import aliases

Every import declares an alias. It exposes public symbols through `Alias.Name`
and hides the original unit path in the importing file:

```pascal id=example-4
program Aliases;

uses Std.Math as Math, Std.Console as Console;

begin
  const Root: real := Math.Sqrt(9.0);
  Console.WriteLn(Root);
end.
```

Here `Sqrt(9.0)` and `Std.Math.Sqrt(9.0)` are errors; use `Math.Sqrt(9.0)`.
Aliases apply to public types, type-qualified enum variants, constants, variables,
routines, and task calls, for example `go Math.Sqrt(9.0)` inside a scope. Record methods and native
built-in type operations keep their existing dot syntax.

Unit names and aliases are case-insensitive. The word `as` is contextual in
`uses`, immediately after the unit name; it remains an ordinary identifier
elsewhere, including qualified unit segments and an alias named `As`.

An alias cannot collide with another alias, an imported or declaring unit namespace root, a built-in
type name, or a declaration in the importing file. Local declarations,
parameters, generic parameters, loop variables, and pattern bindings cannot
shadow it. Rename either the alias or the conflicting declaration.

Import each unit at most once per source file. Different aliases for the same unit
and repeated entries are errors, including differences only in case. Shared
transitive dependencies are allowed. Missing `as Alias` reports FP2022 with an
aliased import example.

Aliases belong to the importing source file; they are not re-exported.
Unit resolution, export restrictions, public type identities, linked symbols,
and compiled-unit sidecars use the original canonical unit name. Renaming an
alias in the program does not rebuild unchanged dependency units.

### Ambiguity rule

Different units may export the same name: `Files.ReadText(...)` and
`Console.ReadText(...)` select their respective explicitly imported units.
Adding another import cannot change either target. An imported short name
reports FP3043 and gives the required alias-qualified spelling. Aliases cannot
be shadowed by local declarations. Built-in operations are independent of
unit imports and their free routine names:

```pascal id=example-5
program Demo;

begin
  const L1: integer := 'hi'.Length();
  const L2: integer := [1, 2].Length();
end.
```

String, array, dictionary, `Option`, and `Result` operations need no `uses`.
The five former type-helper units and their free API have been removed; see
[Dot calls](../language/functions/fluent-calls.md).

## Reserved namespace `Std`

The first segment `Std` (ASCII, any case) is reserved for the standard library. User-defined units use another root segment (for example `MyApp.Utils`).

Intrinsic standard-library entries have registered names such as `Std.Console`
and `Std.Acp.Types`. Trusted source standard-library manifests also export
qualified units, such as `Std.Ai.OpenAi`; only source units listed in that
manifest's `[exports].units` may appear in an application `uses` clause.

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
- direct and transitive dependency interface hashes.

The dependency closure includes nested imported type layouts. Extending a record
therefore rebuilds unchanged transitive consumers whose compiled code or debug
projections use that record. Implementation-only changes still reuse consumers,
and unrelated units remain reusable.

`fpas check`, `fpas run`, and `fpas test` automatically rebuild a missing, stale, corrupt, or
incompatible sidecar. Sources and manifests remain authoritative; `.fpascu` files are replaceable
build products and are excluded from source discovery and formatting.

Before reuse, the compiler limits a complete sidecar to 136 MiB and each semantic-interface or
relocatable-object payload to 64 MiB. It verifies payload hashes, requires the envelope,
interface, and object to identify the same unit, rejects duplicate public symbols under Pascal's
ASCII case-insensitive name rules, and validates register-function metadata, instruction
operands, and references to the optional unit initializer. Functions are encoded separately;
there is no terminal `Halt` instruction in a unit object.
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
Their top-level initialization code is represented by an optional initializer function.
The linker checks that it takes no parameters or captures and returns `unit`, and arranges
startup calls before the program body. Normal completion uses `Return`.

A `const` initializer may call a routine or read an earlier binding. It executes
once in declaration order for each initialized unit. Consumers read exported
computed constants from immutable globals after dependency initialization.
The public interface preserves whether a constant is compile-time known or
computed; calls and references to computed constants cannot be used as `case`
value labels, including through an imported or transitive binding.

Named routine signatures are available throughout the declaring unit, including
mutually recursive siblings. Bodies may reference later values, but calls and
captures must not access bindings before initialization. Interfaces retain
conservative callback dependencies for imported routines and record methods;
the same checks apply when a source-adjacent sidecar is reused. Missing or
unknown callable dependency information cannot prove an initializer safe.
See [declaration order](../language/types/declaration-order.md).

Before returning that image, the linker requires every callable definition to have a matching
function-table entry in the same object, resolves relocations into the final function table,
and applies the complete executable-bytecode validator. Function bodies occupy disjoint
verified instruction ranges; branches and fallthrough cannot escape a function's range.
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

Standard-call dispatch in `crates/fpas-sema/src/check/name_resolution/std_names.rs`
maps an explicitly aliased source name onto its canonical linked identity.
Imported short bindings are never installed. Ordinary local callables retain
their lexical targets. Interface installation and editor lookup enforce the
same alias boundary for native and source units.

Regression coverage is in `crates/fpas-sema/src/tests/expr/std_shadowing.rs` and
`crates/fpas-compiler/src/tests/functions/std_shadowing.rs`.

## See also

- [Visibility](visibility.md)
- [Projects](projects.md)
- [Standard library](../std/README.md)
