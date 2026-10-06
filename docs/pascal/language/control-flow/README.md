# Control flow

Conditionals, loops, and branching.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`if_stmt`, `case_stmt`, `for_stmt`, `for_in_stmt`, `while_stmt`, `repeat_stmt`, `break_stmt`, `continue_stmt`).

| Topic | Description |
|-------|-------------|
| [If / then / else](if-then-else.md) | Conditionals, `elsif` chains, and `end if;` |
| [Case of intro](case-of-intro.md) | `when` arm lists, labels, ranges, and `end case;` |
| [For loops](for-loops.md) | `to` / `downto` counting loops |
| [For-in](for-in.md) | Array and dict iteration |
| [While and repeat](while-repeat.md) | `while` and `repeat … until` |
| [Break and continue](break-continue.md) | Loop control transfer |

Advanced `case` patterns: [Pattern matching](../pattern-matching/README.md).

`case` arms start with `when` and contain scoped statement lists. An optional
final `else` also has its own scope; the statement closes with `end case;`.
Every arm needs a statement, using `null;` for no action.

An `if` branch or loop body contains a nonempty statement list with its own
local scope. Use `null;` for a body that intentionally does nothing. Each
statement ends with `;`; conditionals close with `end if;`, counting and
collection loops with `end for;`, and while loops with `end while;`.
`repeat` keeps `until Condition;`.

A plain `begin ... end;` statement introduces another nested scope. Its local
declarations are visible only inside that block. It may appear inside a
branch or loop, but its `end;` does not close the enclosing control structure.

## See also

- [Pattern matching](../pattern-matching/README.md)
- [Error handling](../error-handling/README.md)
