# Record member visibility

FPAS records currently expose all fields and record routines publicly. Callers can construct and
copy equivalent values, so a library cannot protect a record's internal representation or enforce
invariants through controlled construction.

## Language decision

Extend the existing FPAS `private` and `public` modifiers to record fields, functions, and
procedures. Do not add a separate visibility concept or keyword.

The modifier is written directly before each member, just as it is for unit-level declarations.
There are no visibility sections. A member without a modifier remains public.

```pascal
type
  Counter = record
    private Value: integer;

    private static function CreateWithValue(Value: integer): Counter;
    public static function Create(): Counter;
    function Current(Self: Counter): integer;
    procedure Increment(Self: Counter);
  end;
```

In this example `Current` and `Increment` are public because public is the default.

## Required semantics

- The declaring unit may access private fields and call private record routines.
- Other units may use the public record type in variables, parameters, results, and public APIs.
- Other units may access and call only public members.
- Other units cannot construct a named record containing private fields with a record literal, even
  when every private field has a default value.
- Record updates outside the declaring unit may replace only public fields of an existing value;
  private fields are preserved and cannot be named.
- Public static functions and ordinary public functions may return fully initialized values.
- Compiled-unit interfaces must preserve member visibility and the declaring-unit identity required
  to enforce it after import.

Private members do not make values non-copyable. A caller may still copy or retain a value it
legitimately received. Record member visibility does not introduce borrow checking, lifetime
semantics, or other ownership rules.

## Verification

The language change requires parser, formatter, semantic-analysis, compiled-unit, and compiler
coverage:

- positive tests for same-unit private field access, construction, and private routine calls;
- positive tests for imported public fields and public record routines;
- negative tests for imported private field reads, writes, construction, updates, and calls;
- edge tests proving defaults and explicit type annotations cannot bypass private construction;
- compiled-unit round-trip tests preserving member visibility;
- language and grammar documentation under `docs/pascal/` after implementation.
