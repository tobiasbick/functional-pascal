# Runtime type expressions

The compiler implements the specified typeof(T) expression from
[casts and runtime types](../pascal/language/basics.md#casts-and-runtime-types).
It produces a non-null System.Type for a statically resolved source type. The
[type-inspection example](../../examples/type-inspection/README.md) exercises
primitive, nullable, array, record, and generic parameter types.

```pascal
const NumberType: Sys.Type := typeof(int);
Console.WriteLine(NumberType.FullName);
Console.WriteLine(typeof(int[,]).GetArrayRank());
```

## Type identity

The operand is a type, not a value expression. It uses the existing type resolver
and its accessibility, generic arity/constraint, nullable, and supported-type
rules. Primitive and transparent aliases identify their CLR target. Records,
unions, enums, distinct scalars, and subranges identify their actual emitted
nominal type, not a base primitive or an instance's runtime case subtype.

Closed CLR/source generics, arrays of any supported rank, callable type aliases,
and in-scope generic parameters are supported. Callable aliases identify their
Func/Action delegate type. Reference nullable annotations are erased from CLR
identity, including inside generic/array types: typeof(string?) = typeof(string).
Nullable value types retain actual Nullable<T> identity, so typeof(int?) differs
from typeof(int). T? retains the normal reference/struct constraint requirements.

The operation creates no instance, evaluates no value binding, and does not run
an operand type's static constructor or its containing unit's initializer. A
routine/global initializer containing typeof still follows its own ordinary
execution and initialization rules. Member calls on the resulting Type object
are ordinary .NET calls with their usual behavior.

## Binding and emission

The parser reads a type in parentheses and permits ordinary postfix member/call
syntax on the result. A typed bound node retains that type and the actual
Type.GetTypeFromHandle method imported from reference metadata. Emit ldtoken for
the type followed by a direct call to that method. A valid compiler-generated
type token yields a non-null Type even where the general CLR helper permits an
empty handle; consumers therefore do not need a nullable-result check.

Existing TypeDef/TypeRef/TypeSpec emission supplies concrete, constructed, array,
and !n/!!n generic tokens in the current method/type context. The same path works
inside generic record methods, lexical routines, closures without value captures,
and async state machines. No string-based type lookup, type erasure to object,
generated C#, or additional runtime helper is introduced. Typeof alone adds no
Fpas.Core dependency; source nominal declarations retain their normal metadata
support requirements.

## Limits and verification

Use supported closed type syntax or an in-scope generic parameter. Open generic
definitions, namespaces, source containers, value names, void, and unsupported
byref-like/pointer signatures do not become valid value types through typeof.
The existing syntax/type/access/constraint diagnostics apply. @typeof remains an
escaped identifier rather than an invocation of the reserved form.

The result is not a CLR optional-parameter constant and does not expand the
restricted contract-expression subset. Type-valued custom-attribute arguments
remain part of the separately planned attribute implementation; typeof support
does not claim that source attributes already compile.

Compiler tests check invalid type operands, existing contract/default limits,
deterministic PE/PDB output, real ldtoken/GetTypeFromHandle instructions, and
generic TypeSpec metadata. Runtime tests compare primitive/nullable/array/callable
identities and verify generic type/method/closure/nested/async contexts. Mixed
C#/FPAS SDK projects compare exported Type objects to C# typeof results, check
non-null public annotations with warnings as errors, and verify that type tokens
do not trigger constructors/unit initialization. The contract fixture runs in
Debug and Release; the executable example builds through its actual FPAS project.
