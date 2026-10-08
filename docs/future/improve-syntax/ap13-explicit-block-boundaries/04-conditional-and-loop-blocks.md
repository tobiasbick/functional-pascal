# AP13.4: Conditional, loop, and scoping blocks

Package: [AP13: Explicit block boundaries](README.md)

Status: complete.

## Result

`if`, counted/collection `for`, and `while` bodies are nonempty scoped statement
lists with `end if;`, `end for;`, and `end while;`. `elsif` shares its `if` ending;
`else` followed by `if` starts a nested conditional with a separate ending.
Empty-action bodies use `null;`.

A plain `begin ... end;` introduces an additional scope. `repeat` retains
`until Condition;`; its condition is evaluated in the enclosing scope.
Branch and loop locals remain inside their owning bodies.

## Regression coverage

Parser, scope, compiler, formatter, CLI and editor tests cover branches,
shadowing, nested scopes, loops, empty-action bodies, recovery and execution.
See [control flow](../../../pascal/language/control-flow/README.md).
