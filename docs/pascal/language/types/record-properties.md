# Record properties

Records may declare **computed properties** backed by instance methods. Reading and
writing use normal member syntax; accessors supply the behavior.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`record_property`).

## Visibility

A property in a record declared by a unit is private by default. Write `public`
directly before each property that importing units may read or write:

```pascal
public property Text: string read GetText write SetText;
```

The accessor routines may remain private; the declaring unit uses them to
implement the public property. The `public` modifier is not valid in a
`program` file.

## Declaration

```pascal
type
  Button = record
    function GetText(Self: Button): string;
    procedure SetText(Self: Button; Value: string);

    property Text: string read GetText write SetText;
  end;
```

Version 1 supports read-only, write-only, and read-write forms:

```pascal
property Width: integer read GetWidth;
property Password: string write SetPassword;
property Text: string read GetText write SetText;
```

Rules:

- A property must declare at least one of `read` or `write`.
- `read` names an instance function on the same record with effective signature
  `function Getter(Self: R): T`.
- `write` names an instance procedure with effective signature
  `procedure Setter(Self: R; Value: T)`.
- Accessors may not be static or generic, may not take extra parameters, and must
  match the property type. `Self` and a setter's value parameter are passed by
  value; neither may be declared `mutable`.
- Property names share the case-insensitive member namespace with fields, methods,
  and static routines.
- `read` and `write` are contextual words in the declaration, not reserved
  keywords elsewhere.
- Type aliases expose properties from the resolved record type.

## Reading

A readable property is an ordinary expression of the property type:

```pascal
var Caption: string := Button.Text;
WriteLn(CreateButton().Text);
```

The getter runs once at each access. Postfix chaining may continue from the
result. Reading a write-only property is a compile-time error.

## Writing

Property assignment is recognized when the final member of an assignment target
is a writable property:

```pascal
Button.Text := 'Save';
Form.AcceptButton.Enabled := true;
```

Evaluation order:

1. evaluate the receiver path once;
2. evaluate the right-hand expression once;
3. invoke the setter with the receiver and value.

The receiver before the final property must be a designator. Assignment through
a temporary remains invalid:

```pascal
CreateButton().Text := 'Lost';  // Error
```

Writing a read-only property is a compile-time error.

Property assignment does not require a `mutable` binding on the receiver. The
setter receives `Self` by value and may update data reached through a handle,
registry, or other mutable facility without mutating the handle binding itself:

```pascal
var Button: TuiButton := FindButton();
Button.Text := 'Save';  // Valid — Button is unchanged as a value
```

## Properties versus fields

Properties are behavior, not stored record data:

- record literals cannot initialize a property;
- record update expressions cannot name a property;
- copying a record copies its fields only;
- default field values do not apply to properties.

## See also

- [Records](records.md)
- [Record methods](record-methods.md)
- [Record events](record-events.md) — specialized callable members (not ordinary properties)
- [First-class functions](../functions/first-class.md)
