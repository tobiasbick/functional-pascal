# Records

Records group data in nominal CLR reference types with read-only properties.

```pascal fragment context=declarations id=point
public type Point = record
  public X: double;
  public Y: double;

  public function Length(): double;
  begin
    return Math.Sqrt(self.X * self.X + self.Y * self.Y);
  end function;
end record;
```

A language record is a nominal, sealed CLR **reference type** with read-only
properties. It is not a CLR struct and has no implicit value-copy-on-assignment
operation. Assignment shares the reference; immutability makes sharing safe for
the record's own fields. Referenced objects inside fields may still be mutable.

Fields, instance methods, and static methods are private to the declaring
record by default. `private` spells that restriction explicitly. Use
`public` to expose a member within the record's own accessibility and to
implement an interface member. Code outside the record uses its public API,
including code in the same unit. All fields are read-only.

Construct with `new Point(X := 3.0, Y := 4.0)` or `new Point(3.0, 4.0)`.
Declaration order defines positional constructor order. A field without a default
is required. A field may declare `Name: T := Expression;`; all field types are
explicit. Supplied arguments evaluate in written order, followed by omitted
defaults in declaration order, once per construction. Defaults execute only for
omitted fields. Use named arguments to omit a non-trailing defaulted field.

Defaults use the record's declaration scope and generic parameters, not the
caller's locals. They cannot read self or other instance fields before construction.
They may call routines or read globals under normal initialization rules, and may
throw. A default must type-check for every valid generic instantiation. Defaults
are not implicit CLR parameter constants.
C# callers supply every argument to the all-fields constructor; computed FPAS
defaults are evaluated by FPAS callers before invoking that constructor.

If any field is private, the generated all-fields constructor is private: only
code inside the record can construct it, even if that field has a default.
A public static factory may expose selected construction choices. There is no
custom constructor-body syntax; factories are ordinary methods with optional contracts.
Field defaults do not synthesize additional CLR constructor overloads: satisfying
a CLR `new()` constraint still requires an actual accessible parameterless
constructor, not just an FPAS call with all field arguments omitted.

`P with X := 8.0; end with` constructs a shallow updated record, evaluating `P`
once and updates in written order. It does not clone referenced collections.
Duplicate/unknown/inaccessible update fields are errors. Unmentioned fields,
including private fields, are retained; defaults do not rerun. Generated copy
helpers permit external updates of accessible fields without exposing private
payloads. The operand must be a language record, not an arbitrary imported CLR object.
Record updates are postfix operations. Chain another update or select a member
directly after `end with`.

Instance methods have an implicit read-only `self` binding. `static function` or
`static procedure` declares a type-level method with no self, called through the
record type. Both kinds may be private/public, generic, and have contracts;
static functions may be async under the ordinary rules. Static methods do not
become interface instance implementations. Free functions remain the primary
place for application behavior. There are no user-authored class hierarchies.

For default metadata and constructor/copy emission see
[CLR mapping](../../../specs/clr-mapping.md). Complete examples are in
[record members](record-members.md).

Records generate typed equality, `Equals`, and `GetHashCode`. Corresponding fields
use `EqualityComparer<T>.Default`; array fields therefore compare by reference,
not element-by-element. Equality is not a recursive deep comparison of arbitrary
.NET object graphs. Null equals null; a non-null record never equals null.
Source members may not collide with the constructor, generated equality members,
or emitted property/accessor identities; diagnose these collisions explicitly.

## Example

```pascal
program Shapes;

uses System.Console as Console, System.Math as Math;

type Point = record
  public X: double;
  public Y: double;
end record;

type Shape = variant
  Dot;
  Circle(Radius: double);
  Rectangle(Width: double; Height: double);
end variant;

function Area(Value: Shape): double;
begin
  return case Value of
    when Shape.Dot: 0.0;
    when Shape.Circle(const Radius: double): Math.PI * Radius * Radius;
    when Shape.Rectangle(const Width: double, const Height: double): Width * Height;
  end case;
end function;

begin
  const Start: Point := new Point(X := 1.0, Y := 2.0);
  const Moved: Point := Start with
    X := Start.X + 3.0;
  end with;
  Console.WriteLine(Start.X); // 1
  Console.WriteLine(Moved.X); // 4
  Console.WriteLine(Area(Shape.Rectangle(3.0, 4.0))); // 12
end;
end program;
```

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)
