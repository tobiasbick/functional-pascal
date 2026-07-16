# Static record functions

## Status and priority

Static record functions are the highest-priority language prerequisite for the next Std.Tui2 work. They are a general FPAS language feature, not a special case implemented by the standard library.

The current language supports instance record methods with an implicit receiver supplied for `Value.Method(...)`. It does not support source-defined functions called through a record type. Intrinsic APIs that resemble `Type.New(...)` do not provide this language feature.

No current behavior is specified by this document. The user-facing language reference under `docs/pascal/` must be updated only when the implementation exists.

## Goal

A record may declare a static function inside its type body:

```pascal
type
  TuiRect = record
    X: integer;
    Y: integer;
    Width: integer;
    Height: integer;

    static function Create(
      X: integer;
      Y: integer;
      Width: integer;
      Height: integer
    ): TuiRect;
    begin
      return record
        X := X;
        Y := Y;
        Width := Width;
        Height := Height;
      end
    end;
  end;
```

The function is called through the type:

```pascal
var Bounds: TuiRect := TuiRect.Create(2, 3, 8, 5);
```

## Language contract

- The declaration syntax is `static function` inside a named record body.
- A static function has no implicit receiver and does not declare a `Self` parameter.
- A static function may return its containing record or any other declared return type.
- It is callable through a type designator: `TypeName.FunctionName(Arguments)`.
- It is not callable through a record value. `Value.Create(...)` reports that `Create` is static and must be called through the type.
- Instance methods continue to require `Self: RecordType` as their first parameter.
- Static and instance methods share one case-insensitive name set within the record. Duplicate names are rejected.
- FPAS does not add function overloading. Two static functions in one record must have different names regardless of their parameter lists.
- Static procedures, static fields, constructors with special allocation behavior, and operator overloading are outside this feature.

Static functions are ordinary named functions for return checking, generic function rules, callback compatibility, visibility, and diagnostics. The `static` modifier changes ownership and call resolution; it does not introduce a separate runtime object model.

## Name resolution

For `TuiRect.Create(...)`, semantic analysis resolves `TuiRect` as a type symbol and then looks up `Create` in that record's static function table. The call passes only the explicitly written arguments.

Resolution must work with:

- a local record type;
- an imported record type available by short name;
- a fully qualified record type;
- a public type alias whose resolved type is a record from a private source-library unit;
- case-insensitive type and function spelling.

The fully qualified callable identity remains `ResolvedRecordType.Create`. A public alias does not duplicate the implementation.

## No overload-based constructors

Construction APIs use distinct semantic names:

```pascal
TuiPoint.Create(X, Y)
TuiSize.Create(Width, Height)

TuiRect.Create(X, Y, Width, Height)
TuiRect.FromEdges(Left, Top, Right, Bottom)
TuiRect.FromPointSize(Position, Size)
TuiRect.FromCorners(TopLeft, BottomRight)
```

`Create` constructs a value directly from its stored representation. `From...` names are reserved for conversion from another representation. Copying a record does not require a static function because records have value semantics:

```pascal
var Copy: TuiRect := OtherRect;
```

## Implementation map

### Lexer and grammar

- Add the `static` keyword and token display text.
- Extend the record-member grammar to accept `static function`.
- Reject `static` outside supported record function declarations with a targeted diagnostic.

### Parser and AST

- Represent whether a record function is instance-bound or static explicitly in the AST.
- Keep instance procedures and functions compatible with the existing representation where practical.
- Preserve the modifier through formatting and source-span reporting.

### Semantic analysis

- Store static functions separately from instance methods, or attach an explicit receiver kind to each method entry.
- Validate that static functions do not declare implicit `Self` semantics.
- Resolve calls whose receiver designator denotes a type symbol.
- Reject static calls through values and instance calls through types with specific hints.
- Preserve the existing no-overload and duplicate-name rules.
- Support record aliases without cloning or recompiling the static function.

### Compiler and bytecode

- Compile a static record function under its qualified callable name.
- Compile `TypeName.Function(...)` like a normal named call without emitting a receiver value.
- Keep instance-method lowering unchanged: it still emits the receiver before explicit arguments.
- No new bytecode instruction is required unless implementation work proves otherwise.

### Source-library linker

- Rewrite parameter types, return types, nested declarations, and bodies of static record functions.
- Preserve qualified callable identity when a public facade aliases a record from a private implementation unit.
- Ensure private implementation units remain inaccessible through `uses` even though their aliased public types expose static functions.

### Formatter and documentation

- Format `static function` consistently with existing record functions.
- Add the implemented syntax to the grammar and record/function language reference.
- Document static and instance call differences with diagnostics and examples.

## Required tests

- Parser acceptance and AST shape for a static record function.
- Parser recovery for unsupported `static` placements.
- Semantic success for local, imported, qualified, and aliased record types.
- Semantic rejection of `Self`-style misuse, calls through values, duplicate names, and overload attempts.
- Compiler and VM execution proving that only explicit arguments are passed.
- Return-type and generic-function checks identical to ordinary functions.
- Formatter input/output and idempotence coverage.
- Source-library integration using a public facade alias over a private implementation record.

## Completion criteria

The feature is complete when the documented syntax works for ordinary projects and source standard-library units, the full test matrix passes, the language reference describes the implemented behavior, and Std.Tui2 can remove its free geometry factory functions.
