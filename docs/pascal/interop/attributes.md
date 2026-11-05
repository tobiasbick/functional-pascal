# CLR attributes

Annotate source declarations with actual .NET attribute classes. Attributes
retain their CLR target and constant representation; they do not introduce a
separate FPAS annotation runtime.

```pascal fragment context=declarations
type Measurement = record
  [Serialization.JsonPropertyNameAttribute('sample_name')]
  Name: string;
end record;
```

## Targets and declaration scope

An attribute list contains one exact type name per pair of brackets. Multiple
lists may precede a declaration, before public/private/static/async modifiers.
There is no implicit Attribute suffix lookup. Source declarations can use
accessible concrete attribute classes from the supplied references, including
closed generic attribute classes. FPAS does not add class inheritance or a new
way to author CLR attribute subclasses.

| Source declaration | Emitted attribute target |
| --- | --- |
| Record or union | Its CLR class definition |
| Interface | Its CLR interface definition |
| Enum | Its CLR enum definition |
| Distinct scalar or subrange | Its actual wrapper class/struct definition |
| Free routine, record method, or interface method | Its declared CLR method |
| Named nested routine | Its emitted closure method |
| Async routine | Its kickoff method, retaining compiler-owned state-machine metadata |
| Record field | Its CLR property, excluding backing field, getter, and constructor parameter |
| Interface property | Its CLR property, excluding the getter |

Public/private accessibility does not change the target mapping. Transparent
aliases and globals are not attribute targets in this baseline. Unit/program
headers, enum/union cases, lambdas, parameters, returns, assemblies, and explicit
backing-field targets have no source attribute syntax.

Read AttributeUsage from actual reference metadata, including inherited usage.
Check the CLR target mask and AllowMultiple. Duplicate non-repeatable attributes
are errors. A field-only attribute cannot annotate a source record property.
Compiler-owned Core and nullable metadata cannot be overridden. A record or
interface property annotation also cannot contradict its declared nullable type: for
example, use string? rather than MaybeNull on a string property. This preserves
the declared contract when another FPAS assembly imports its metadata.

## Arguments and binding

Constructor arguments are positional. Use the ordinary constructor overload,
conversion, optional-parameter, and params rules. Following Name := Value items
set public writable instance fields/properties; they are not constructor labels.
No positional argument may follow a named assignment. Names are case-sensitive;
duplicate, missing, static, readonly, indexed, and inaccessible named members are
diagnosed, including hidden base members.

Legal value types are bool, char, sbyte/byte, short/ushort, int/uint, long/ulong,
float/double, string, System.Type, CLR enums, boxed legal values through object,
and one-dimensional arrays of legal element types. Explicitly typed nullable
references follow ordinary argument conversion rules. Null strings, Type values,
and arrays use their CLR custom-attribute encodings. Boxed arrays and enums retain
their actual payload type rather than becoming strings or erased integers.

Values are representable compile-time constants, typeof of supported closed
types, or array literals of those values. Constant evaluation uses the existing
checked/unchecked scalar rules and invariant literal char/string concatenation.
Array allocation expressions, arbitrary calls/properties, object construction,
delegates, decimal/native integers, nullable value wrappers, and scalar domains
other than actual enums are not attribute constants. Global const initializers
are runtime fields, not compile-time attribute constants.

Typeof arguments retain real CLR identity for aliases, primitive/array/nullable
types, closed generics, and source nominal types. An argument containing an
enclosing type/method parameter cannot be serialized into attribute metadata and
is rejected. The compiler does not substitute a display name or object for it.

## CLR output and pseudo-attributes

Bind attributes to typed constructor and named-member values. A shared overload
selector supplies the same argument matching as normal calls, without creating
runtime argument temporaries. Emit ECMA-335 CustomAttribute blobs and actual
constructor MemberRefs, including generic owner signatures. Type/enum names use
their emitted nested/generic/assembly identities; the CLR can resolve them when
reflection reads the attribute.

No referenced attribute constructor, setter, or assembly code executes during
compilation. Those constructors/setters execute only when normal .NET consumers
instantiate the attributes. Source declaration annotations are not copied onto
compiler-generated getters, backing fields, async MoveNext bodies, or helpers.
Comments before the first attribute continue to supply Markdown/XML documentation;
comments within the modifier sequence do not replace them. PE/PDB output remains
deterministic and independent of XML generation.

SerializableAttribute maps to the CLR Serializable type flag rather than an
ordinary CustomAttribute row. On a union it also marks the generated case types.
MethodImplAttribute maps supported inlining, optimization, and synchronization
options to the declared method's implementation flags. Its code type remains
managed IL; on async methods these flags apply to the kickoff method.

Native/marshalling/layout pseudo-attributes requiring unsupported CLR structural
changes are rejected, including DllImport, StructLayout, FieldOffset, MarshalAs,
ComImport, PreserveSig, and NonSerialized. They must not be emitted as ineffective
custom blobs. Source attributes do not add native interop, source field-layout
control, or attribute-driven compiler features such as Conditional call omission
or Obsolete use-site warnings. Ordinary emitted attributes remain available to
other .NET tools and languages.

## See also

- [Complete JSON example](../libraries/json.md#example)
- [Topic overview](README.md)
- [CLR mapping](../../specs/clr-mapping.md)
- [Test coverage](../../../tests/coverage.md)
