# Record events

Records may declare **events**: specialized computed members whose logical type is a
function or procedure handler. Storage is always behind `Option of Handler` accessors;
the public assignment syntax hides that `Option`.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`record_event`).

## Visibility

An event in a record declared by a unit is private by default. Write `public`
directly before each event that importing units may assign, clear, or inspect
with `Assigned`:

```pascal
public event OnClick: ClickHandler read ReadOnClick write WriteOnClick;
```

The accessor routines may remain private. A visible event still may be raised
only by its declaring unit. The `public` modifier is not valid in a `program`
file.

## Declaration

```pascal
type
  ClickHandler = procedure(Sender: Button);

  Button = record
    function ReadOnClick(Self: Button): Option of ClickHandler;
    procedure WriteOnClick(Self: Button; Handler: Option of ClickHandler);

    event OnClick: ClickHandler read ReadOnClick write WriteOnClick;
  end;
```

Rules:

- `HandlerType` must be a function or procedure type (including a type alias of one).
- Both `read` and `write` are required in version 1.
- The getter is an instance function with signature
  `function Getter(Self: R): Option of HandlerType`.
- The setter is an instance procedure with signature
  `procedure Setter(Self: R; Value: Option of HandlerType)`.
- Accessors may not be static or generic, may not take extra parameters, and must not
  use `mutable` parameters.
- Event names share the case-insensitive member namespace with fields, methods, static
  functions, and properties.
- `read` and `write` are contextual words in the declaration.
- Type aliases expose events from the resolved record type.

Events are not ordinary [record properties](record-properties.md): they are not readable
as values, use `nil` / `Assigned`, and may be raised only from the declaring unit.

## Assignment

```pascal
B.OnClick := HandleClick;           { setter receives Some(HandleClick) }
B.OnClick := Controller.HandleClick;
B.OnClick := procedure(Sender: Button) begin … end;
B.OnClick := nil;                   { setter receives None }
```

Rules:

- A compatible named routine, bound method, or closure is wrapped in `Some` and passed
  to the setter.
- `nil` is accepted only as the right-hand side of an event assignment (not for ordinary
  function variables). Use `None` for `Option` values.
- Assignment replaces the previous handler synchronously.
- The receiver is evaluated once before the handler expression.
- Assignment through an immutable handle binding is valid (same as properties).
- Assignment through a temporary receiver remains invalid.

## `Assigned`

`Assigned` is a language builtin (not `Std.*`):

```pascal
if Assigned(B.OnClick) then
  …
```

It evaluates the getter once and returns whether the result is `Some`. Reading an event
in any other value context is a compile-time error.

## Raising

Only the unit that declares the event may invoke it. External code may assign, clear, or
test a visible event but cannot raise it.

```pascal
if Assigned(B.OnClick) then
  B.OnClick(B);
```

Invocation evaluates the getter once, unwraps the handler, and calls it synchronously.
Raising an empty event is a runtime error. Program-local events may be raised only from
program-scoped code.

An event raise cannot be spawned with `go`: the installed handler may own mutable
captures and therefore be task-bound. Raise the event synchronously on its current task;
the handler itself may start explicitly safe work when needed.

## Events versus fields and properties

- record literals cannot initialize an event;
- record update expressions cannot name an event;
- copying a record copies its fields only;
- bare event reads are forbidden outside `Assigned`, assignment, and owner raise.

## See also

- [Record properties](record-properties.md)
- [Record methods](record-methods.md) — bound methods as handler values
- [First-class functions](../functions/first-class.md)
- [Closures](../functions/closures.md)
- [Result and Option types](result-option-types.md)
