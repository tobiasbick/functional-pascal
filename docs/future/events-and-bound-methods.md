# Events and bound record methods

## Status

Two implementation milestones:

1. **Bound record methods — done.** Spec:
   [record-methods.md](../pascal/language/types/record-methods.md#bound-methods-as-values),
   [first-class.md](../pascal/language/functions/first-class.md). Depends on
   [capturing closures](../pascal/language/functions/closures.md).
2. **Event properties — planned.** Depends on implemented
   [record properties](../pascal/language/types/record-properties.md).

Current next language step for Tui2: Milestone 2 (event properties) in this plan.

## Goals

Provide a Pascal-shaped event model that supports named routines, bound record methods, and
capturing closures without introducing objects, inheritance, hidden event fields, or a general
publish/subscribe system.

```pascal
Button.OnClick := HandleClick;
Button.OnClick := Controller.HandleClick;
Button.OnClick :=
  procedure(Sender: TuiButton)
  begin
    Count := Count + 1
  end;
Button.OnClick := nil;
```

## Milestone 1: bound record methods

**Status: implemented.** User-facing docs live under `docs/pascal/` (links above). The rules
below remain the design record for Milestone 2.

Reading an instance method without calling it creates a callable value with the receiver bound:

```pascal
type
  Counter = record
    Base: integer;

    function Add(Self: Counter; Value: integer): integer;
    begin
      return Self.Base + Value
    end;
  end;

var C: Counter := record Base := 10; end;
var AddTen: function(Value: integer): integer := C.Add;
```

`C.Add` evaluates `C` exactly once and captures its value at the point of binding. Calling
`AddTen(5)` invokes `Counter.Add(C, 5)`.

Rules:

- The method reference must resolve to one instance function or procedure.
- The resulting callable omits the implicit `Self` parameter.
- The receiver is captured by value. Later assignment to the source variable does not change it.
- A handle record remains useful because copying it preserves the identity encoded by the handle.
- A static function is already a normal named callable and is not a bound method.
- A procedure method produces a procedure value; a function method produces a function value.
- Generic arguments must be inferable from the receiver and expected callable type.
- Visibility is checked when the method is bound.
- The receiver and every later argument are evaluated exactly once.

If mutable receiver semantics are added separately, binding a method with mutable `Self` must not
silently capture a copy and pretend to mutate the original. Version 1 rejects that binding and
suggests a closure that explicitly captures a `mutable var`.

## Milestone 2: event properties

An event is a specialized computed property whose logical type is an ordinary function or procedure
type. It stores no handler inside the visible record value.

```pascal
type
  TuiClickHandler = procedure(Sender: TuiButton);

  TuiButton = record
    function ReadOnClick(Self: TuiButton): Option of TuiClickHandler;
    procedure WriteOnClick(
      Self: TuiButton;
      Handler: Option of TuiClickHandler
    );

    event OnClick: TuiClickHandler read ReadOnClick write WriteOnClick;
  end;
```

The accessors may store the optional handler in a registry, reference-owned state, or another
implementation chosen by the declaring unit. For Tui2 handles they resolve the canonical live
registry entry.

The declaration is based on the property machinery from
[record-properties.md](../pascal/language/types/record-properties.md), with additional callable and ownership rules. Event
members are not allowed as ordinary record fields.

## Event declaration rules

An event declaration has this form:

```text
event Name: HandlerType read Getter write Setter;
```

Rules:

- `HandlerType` must resolve to one function or procedure type.
- Both accessors are required in version 1.
- The getter is an instance function returning `Option of HandlerType`.
- The setter is an instance procedure accepting `Option of HandlerType` after `Self`.
- The accessors belong to the same record and follow normal property visibility and resolution.
- Event names share the record's case-insensitive member namespace.
- Type aliases expose events from their resolved record type.
- Event accessors may validate live handles and report the normal registry diagnostics.

The public assignment syntax hides the storage `Option`:

```pascal
Button.OnClick := HandleClick;  { setter receives Some(HandleClick) }
Button.OnClick := nil;          { setter receives None }
```

The `nil` literal in version 1 is accepted only on the right-hand side of event assignment or where
another implemented nullable type explicitly permits it. Ordinary function and procedure variables
do not become implicitly nullable.

## Assignment and `Assigned`

Event assignment is specialized property assignment:

- a compatible named routine, bound method, or closure is wrapped in `Some` and passed to the setter;
- `nil` is lowered to `None` and clears the handler;
- assignment replaces the previous handler synchronously;
- the receiver is evaluated once before the handler expression;
- assignment through an immutable handle binding is valid because the binding itself is unchanged;
- assignment through a temporary receiver remains invalid in version 1.

`Assigned(Button.OnClick)` calls the getter exactly once and reports whether it returned `Some`.
Reading an event in any other value context is forbidden. In particular, application code cannot
copy the current handler out of an event or bypass its setter.

## Raising events

Only the unit that declares an event property may invoke it. External code may assign, clear, or
test a visible event but cannot raise it.

Inside the owning unit, invocation uses ordinary Pascal call syntax:

```pascal
if Assigned(Button.OnClick) then
  Button.OnClick(Button);
```

Invocation evaluates the getter once, unwraps the handler, and calls it synchronously. Directly
invoking an empty event is a runtime error. Owning libraries normally check `Assigned` and define
their own empty behavior. Tui2 procedure events do nothing when empty, and its empty boolean raw
input events are treated as `false` by routing code.

The declaring unit may expose a focused internal raising routine when dispatch ownership belongs in
a sibling implementation unit. That routine invokes the event; it does not make external event
invocation legal.

## Handler execution

- Each event has zero or one handler.
- Replacement or clearing releases the old closure when no other callable owns it.
- Invocation runs synchronously on the caller's task.
- Panics propagate through the event call unchanged.
- Replacing or clearing an event inside its current handler affects only later invocations.
- Recursive invocation is permitted by the language; a library may reject re-entry for a specific
  event with a documented diagnostic.

Multicast subscription, subscription tokens, weak subscriptions, topic strings, and unspecified
delivery order are deliberately absent.

## Interaction with closures and bound methods

These assignments are equivalent at the event boundary when their signatures match:

```pascal
Button.OnClick := NamedHandler;
Button.OnClick := Controller.HandleClick;
Button.OnClick := procedure(Sender: TuiButton) begin Handle(Sender) end;
```

The event setter owns one copy of the callable. Capture lifetime and task-transfer restrictions come
from [closures.md](../pascal/language/functions/closures.md). Assigning a task-bound closure is valid when the
event is raised on that same task.

## Diagnostics

Diagnostics must identify the record, event, or method and include a correction for:

- incompatible handler parameters or function result;
- attempting to assign a called result instead of a callable;
- invoking an event outside its declaring unit;
- reading an event outside `Assigned`, assignment, or owner invocation;
- missing or incompatible `Option` accessors;
- assigning through a temporary receiver;
- binding an unresolved, static, or unsupported mutable-receiver method;
- serializing, comparing, or using an event in record data operations;
- transferring a handler whose captures are task-bound.

## Expected implementation shape

```text
crates/fpas-parser/src/
  ast/types/record_event.rs           — NEW: event property declaration
  parser/types/record_event.rs        — NEW: event declaration parsing
  tests/types/record_events.rs        — NEW: parser and recovery coverage

crates/fpas-sema/src/check/
  expr/bound_method.rs                — NEW: method-value resolution
  decl/types/record_events.rs         — NEW: handler and accessor validation
  stmt/event_assignment.rs            — NEW: handler wrapping and ownership
  expr/event_access.rs                — NEW: Assigned and owner-only invocation

crates/fpas-compiler/src/compiler/
  expr/bound_method.rs                — NEW: receiver capture
  stmt/event_assignment.rs            — NEW: Some/None setter lowering
  expr/event_access.rs                — NEW: getter, Assigned, and invocation

crates/fpas-fmt/src/emit/
  record_event.rs                     — NEW: event declaration formatting
```

Reuse closure environments, `Option`, property accessor metadata, and ordinary callable invocation.
Do not add an event-slot field to record values, duplicate closure invocation in Tui2, or introduce
an event-specific VM object when the accessors and existing managed values already provide the
required behavior.

## Implementation order

### Bound-method milestone

1. ~~Complete the capturing-closure acceptance criteria.~~
2. ~~Add bound method AST interpretation, sema resolution, and lowering.~~
3. ~~Add receiver lifetime, evaluation-order, and callable-compatibility tests.~~
4. ~~Update current language documentation for bound methods.~~

### Event-property milestone

5. ~~Complete the record-property acceptance criteria.~~
6. Add event declaration parsing, formatting, and accessor validation.
7. Add assignment wrapping, `nil` clearing, and `Assigned`.
8. Add owner-only invocation and empty-event diagnostics.
9. Add cleanup, replacement, panic, and task-bound tests.
10. Migrate one small non-TUI canary and then the Tui2 event surface.
11. Update grammar and current documentation only after event properties work.

## Required tests

- bound function and procedure methods;
- receiver captured exactly once and by value;
- bound method accepted anywhere its callable type is expected;
- named routine, bound method, and anonymous closure assigned to one event;
- replacement and `nil` clearing through property accessors;
- copied handle records resolve the same registry-backed handler;
- owner unit can invoke while external units cannot;
- `Assigned` distinguishes installed and empty handlers;
- invoking an empty event directly produces the documented diagnostic;
- handler released on replacement, clear, registry destruction, and panic cleanup;
- handler replacing or clearing itself;
- precise signature, ownership, temporary-receiver, and transfer diagnostics;
- formatter, project linking, and source maps preserve event and bound-method expressions;
- no event storage is added to record copy or equality semantics.

## Acceptance criteria

- all syntax examples compile and run with deterministic single-handler behavior;
- event state is owned by accessors rather than copied record fields;
- only the declaring unit can raise an event;
- bound receivers and closure environments have defined lifetimes;
- no object model, multicast bus, or hidden event-field aliasing is introduced;
- properties, events, formatter, linker, bytecode lowering, VM, docs, and tests agree;
- complete workspace and FPAS regression verification passes.

## Plan lifecycle

Keep this document until both milestones are documented under `docs/pascal/` and Tui2 no longer
depends on hypothetical behavior. Mark the bound-method milestone complete independently if it lands
before record properties. Remove the plan only after event properties are complete.
