# Type argument inference

Generic calls infer type arguments from supplied values. This applies to
[generic routines](generic-routines.md) and [generic data types](../types/generics.md)
without changing their CLR representation or declaration syntax. Bindings,
parameters, lambda signatures, and routine results still require explicit types.

```pascal fragment
function Identity<T>(Value: T): T;
begin return Value; end function;

const Number: int := Identity(42);
const Copy: function(int): int := Identity<int>;
```

## Routine and CLR method calls

Inference applies to free FPAS routines, instance/static record methods, imported
FPAS routines, and supported CLR generic methods. Explicit calls remain valid.
The [pipeline](../../../../examples/generic-pipeline/README.md) uses Compose(Double,
Label) and Map(Items, Format); a method group for generic Identity keeps its
explicit arguments.

Evidence comes from argument types, array elements, matching constructed generic
types, and typed callable signatures. A List<string> or string[] can provide the
string argument of an IEnumerable<T> parameter. A typed lambda can provide both
the input and output of Func<T, U>, including when it converts to a custom CLR
delegate. Unique nongeneric method groups provide their ordinary callable types.

The solver gathers exact/lower/upper bounds according to invariant, covariant,
and contravariant positions and requires a unique compatible supplied bound type.
It uses implemented implicit conversions, including numeric widening. It does
not invent object as a fallback for unrelated inputs or choose arbitrarily
between multiple instantiations of the same interface. Stored arrays retain
normal CLR conversions. An array literal can instead construct the vector element
type selected by its parameter, such as a double[] for a widened numeric call.

Constraints are checked after inference and never supply missing type arguments.
The expected routine result does not infer routine arguments. For example, a
typed destination does not make Create<T>() infer T from its result; that call
still needs Create<int>(). Generic method groups also retain explicit type
arguments; a delegate destination alone does not close Identity<T>.

Imported overloads are inferred independently before ordinary applicability,
constraint, and better-member checks. Equivalent closed parameter lists prefer
a nongeneric member, or the more specific generic parameter structure where
applicable. Ambiguous candidates remain an error. A target result never selects
an overload and there is no dynamic/reflection fallback.

## Union constructors and expected types

The uninstantiated name of a generic union supports case construction:

```pascal fragment
type Tree<T> = union
  Empty;
  Leaf(Value: T);
  Branch(Left: Tree<T>; Right: Tree<T>);
end union;

const Leaf: Tree<int> := Tree.Leaf(42);
const Empty: Tree<int> := Tree.Empty;
const Branch: Tree<int> := Tree.Branch(Tree.Empty, Tree.Leaf(7));
```

Payload types and a matching expected union type supply evidence. Bindings,
returns, closed routine parameters, record fields, and typed array elements can
supply that expected type. A fieldless case, or a parameter absent from a case's
payloads, requires enough expected context. Other payloads can establish the type
for a sibling Tree.Empty. This also works through an imported unit/container and
through checked union metadata from another FPAS project.

The expected union instantiation is exact. Tree.Leaf(1) can initialize Tree<double>
by converting the payload to double. It cannot silently change a Tree<int>
destination to Tree<string>. Null is never a union case. Named payloads keep their
labels, and arguments execute once in written order even when their labels reorder
the constructor's parameter slots.

The owner must identify one generic union definition. Ambiguous imported generic
arities require explicit arguments. The shorthand applies only to actual FPAS
unions validated through their metadata contract. Records and ordinary CLR
generic classes still require arguments on type references, new expressions, and
static type receivers. Tree<int>.Leaf(42) and Tree<int>.Empty remain available;
type arguments belong on the owner, not on the nongeneric case factory.

## Nullability and contextual arguments

Nullable annotations participate in the inferred types. Identity(Value) preserves
a nullable string variable's type and cannot initialize a non-null string without
validation. A null literal supplies no underlying type by itself. Alongside a
reference bound it can make the result nullable; it does not invent Nullable<int>
from an int argument plus null. A T? parameter with a struct/reference constraint
can infer the underlying T from a typed nullable argument.

Empty array literals and cases lacking sufficient payload evidence can wait for
other arguments to establish their closed target. Typed array literals similarly
give contextual elements the type established by their other elements. Binding
does not execute, duplicate, or reorder the generated argument expressions.
Nullable analysis conservatively drops mutable captured facts that could be
invalidated by an argument awaiting context.

This remains local call inference. It does not propagate expected return types
backwards through arbitrary calls or implement a whole-program type solver.
If a conditional/case expression or overloaded method group cannot supply its
own type in an otherwise untyped argument position, use a typed intermediate
binding or explicit call arguments. Fully general nested generic type paths
remain a tracked [baseline gap](../../../future/language-gaps.md#finish-the-existing-baseline-first).

## See also

- [Generic routines](generic-routines.md)
- [Generic data types](../types/generics.md)
- [Test coverage](../../../../tests/coverage.md)
