# Bindings

`const` binds a value once. `var` creates writable storage. Both accept computed
initializers, evaluated once when the declaration is reached. Repeat the keyword
for each binding; grouped declarations are invalid.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`const_block`,
`var_block`, `const_stmt`, `var_stmt`).

```pascal
const Name: string := 'Alice';
var Age: integer := 30;
```

Unit/program declarations require an explicit type. Local bindings may infer
their type from the initializer. An annotation constrains that initializer; later
uses and assignments never determine a binding's type.

```pascal
function NextAge(Current: integer): integer;
begin
  const Next := Current + 1;
  var Result := Next;
  Result := Result + 1;
  return Result;
end function;
```

A const binding cannot be reassigned, and its stored fields, array elements and
dictionary entries cannot be replaced. Imported bindings have the same rule;
their declared alias preserves the original binding's permissions.

Assigning or passing records and collections copies their values. Changes to a
separate var binding preserve earlier snapshots, including nested collections.
Resource handles retain their resource identity, and stateful closure copies share
their mutable capture environment.

```pascal
procedure Demonstrate();
begin
  const Original := [[1, 2]];
  var Copy := Original;
  Copy[0][1] := 9;
  // Original[0][1] remains 2.
end procedure;
```

Caller mutation uses an explicit [var parameter](../functions/var-parameters.md)
and a `var` argument. A value parameter is a read-only snapshot; create a local var
copy when only the routine's own value needs to change.

## See also

- [Constants and static expressions](constants.md)
- [Local inference](local-variables.md)
- [Records and value copying](../types/records.md#immutability)
