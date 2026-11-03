# Keywords and reserved names

Keywords use lowercase spelling and are case-sensitive. Prefix a keyword with
@ when naming a foreign CLR member that uses the same spelling.

```text
async await begin break case catch const continue discard distinct div do downto else
elsif end ensures enum false finally for function if implements in is mod new not null
of or out params private procedure program public raise range record ref repeat return
requires shl shr static subscribe then to true try type union unit unsubscribe until uses using var
when where while with and xor
```

`as` is also a keyword, used in imports and type patterns. `class` and `struct`
are constraint keywords. `cast`, `typeof`, and `unchecked` are reserved
compiler-recognized forms described by [casts and runtime types](../language/basics/conversions.md); escape them with `@` to refer
to foreign identifiers with those names. `self` and `args` are reserved implicit bindings in
their applicable scopes. There are no predefined `integer`, `real`, `go`, `dict`,
`channel`, or task-type keywords.

## See also

- [Topic overview](README.md)
- [Handbook](../README.md)
- [Syntax grammar](../../specs/syntax.md)
