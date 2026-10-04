# Constants and static expressions

`const` creates an immutable binding. Its initializer may call routines, construct
values or make a decision; it runs once when the declaration is reached.
Unit/program bindings require a type annotation, while local bindings may infer it.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`const_block`,
`const_def`, `const_stmt`).

```pascal
function Compute(): integer;
begin
  return 42;
end function;

const Answer: integer := Compute();
```

Immutability does not imply a compile-time value. Literal/operator expressions and
references to static const bindings qualify for static contexts. Ordinary routine
calls and reads of var bindings remain non-static. Static labels and range endpoints
reject computed values and report F2014; use a guard for a runtime comparison.

Reached static integer operations report F2020 for overflow or a zero divisor
at the operation. The evaluator visits collection/record components, aggregate
comparisons and membership expressions, as well as static defaults, labels and
range endpoints. Lazy Boolean evaluation skips an unneeded operand:
`([1] = [2]) and (1 div 0 = 0)` evaluates to false.
Routine calls remain non-static; failures in their bodies occur when executed.
Real division by zero follows the [IEEE arithmetic rules](operators.md).

Intrinsic standard constants, including `Math.Pi` and Console color/mode values,
participate in the same static evaluation after alias resolution. For example,
`([Math.Pi] = [Math.Pi]) and (1 div 0 = 0)` reports F2020, while replacing
`and` with `or` skips the invalid operand. Static exports and record defaults
retain these values across compiled-unit reuse.

```pascal
const Lower: integer := 1 + 1;
const Upper: integer := Lower + 3;

procedure Demonstrate(Value: integer);
begin
  const Expected := Compute();
  case Value of
    when Lower..Upper: null;
    when const Candidate if Candidate = Expected: null;
    else null;
  end case;
end procedure;
```

Static constant classification follows lexical resolution: a local shadow has its
own value, and leaving that scope restores the enclosing binding. A computed const
export remains immutable runtime storage across compiled-unit interface reuse;
scalar static exports preserve their compile-time values. Static records and
collections retain static classification and use immutable global storage when
imported. Their interfaces also retain evaluated data for static comparisons,
membership and lazy Boolean guards across artifact reuse. Their runtime value
copies follow the ordinary snapshot rules.

A procedure value may initialize a const binding. A procedure call produces no
value and cannot initialize any binding.

## See also

- [Bindings](variables.md)
- [Local inference](local-variables.md)
- [Patterns](../pattern-matching/README.md)
