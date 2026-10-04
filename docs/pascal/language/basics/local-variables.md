# Local bindings and inference

Local `const` and `var` bindings may omit their type annotation. Inference uses the
initializer alone, including ordinary calls, constructors, callable values and
value-producing `if`/`case` decisions. It never reads later assignments or infers a
routine's parameter/result types from its body.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`const_stmt`,
`var_stmt`).

```pascal
function FullName(First: string; Last: string): string;
begin
  const Space := ' ';
  const Name := (First + Space) + Last;
  return Name;
end function;
```

The initializer must determine one complete type. Empty collections, payloadless
generic variants and unspecialized generic callable values need an annotation.
Incompatible decision branches are errors; an annotation supplies expected type
context without forcing an incompatible conversion.

```pascal
procedure Demonstrate();
begin
  const Values: array of (integer) := [];
  const Empty: Option of (string) := Option.None;
  var Count := 0;
  Count := Count + 1;
end procedure;
```

Bindings follow ordinary lexical scope and sequential visibility. Named nested
routines may capture enclosing locals with inferred types; their own signatures
remain explicit. Immutable captures are snapshots, while var captures share a
mutable cell. Editor hover, field completion and callable signatures use the same
checked inferred binding type.

## See also

- [Bindings](variables.md)
- [Constants and static expressions](constants.md)
- [Closures](../functions/closures.md)
