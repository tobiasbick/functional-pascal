# 02 – Seed catalog

Seed entries for the [foreign-habit diagnostics plan](README.md), collected as described in
[01-catalog-method.md](01-catalog-method.md). Field meanings are defined there.

**Probe:** each habit was written as a minimal program and checked with `fpas check` on the
current tree (2026-10-15). "Today" quotes the first diagnostic; "+N" counts further errors
reported for the same file. Proposed wording is a draft and is settled per package in
[05-rollout.md](05-rollout.md).

## Summary

| Rating | Count | Meaning |
| --- | --- | --- |
| good | 9 | Specific, correct hint at the right place; at most minor polish |
| partial | 8 | Right place, but hint is generic, incomplete or followed by cascades |
| poor | 20 | Generic or misleading diagnostic, or a long cascade |
| deferred | 1 | Not detectable at compile time |

Two probes were valid FPAS and are not habits: Pascal `repeat … until` and Ada `null;`.
`for … downto` is also valid; its probe only failed on an empty loop body.

## Declarations

| ID | Habit | Origin | Today | Rating | Proposed | Stage |
| --- | --- | --- | --- | --- | --- | --- |
| H-DECL-01 | `var` section with several declarations | Pascal | FP2001 "Expected `:=`, found `;`" at the first line, +4; FP2015 only at the second line | partial | Report FP2015 once at the section, mention both "repeat `var`" and "initialize with `:=`" | parser |
| H-DECL-02 | `const N = 3;` | Pascal | FP2001 "Expected `:`, found `=`", +7 | poor | "Pascal `const N = 3` – FPAS writes `const N: integer := 3;`", no cascade | parser |
| H-DECL-03 | `X : integer := 1;` without `var` | Ada | FP2015 with hint `var X: integer := 0;` | good | Keep; reuse the actual initializer instead of `0` | parser |
| H-DECL-04 | `type PInt = ^integer;` | Pascal | FP1001 "Unexpected character `^`" | poor | "FPAS has no pointers; use records, arrays, `Option of T`" | lexer/parser |
| H-DECL-05 | `type TFoo = class … end;` | Delphi | FP2001 "Expected `;`, found `end`", +2 | poor | "FPAS has no classes; declare a `record … end record;` and functions on it" | parser |
| H-DECL-06 | `unit U; interface … implementation … end.` | Delphi | FP2001 "Expected `end unit;`, found `interface`", +1 | partial | "FPAS units have no interface/implementation sections; mark exports `public` and close with `end unit;`" | parser |
| H-DECL-07 | `procedure Q;` without parentheses | Pascal | FP2001 "Expected `(`, found `;`", +9 | poor | "Routines always declare a parameter list: `procedure Q();`", no cascade | parser |
| H-DECL-08 | `function F return integer is … end F;` | Ada | FP2001 "Expected `(`, found `return`", +11 | poor | "FPAS writes `function F(): integer; begin … end function;`" | parser |
| H-DECL-09 | `char` type | Pascal | FP3001 "Unknown type `char`" | partial | "FPAS has no character type; one Unicode scalar is a `string`" | sema |
| H-DECL-10 | `TList<Integer>` | Delphi | FP2001 "Type applications use `of`, not angle brackets", +2 | good | Keep; remove the two follow-up errors | parser |

## Routines and calls

| ID | Habit | Origin | Today | Rating | Proposed | Stage |
| --- | --- | --- | --- | --- | --- | --- |
| H-CALL-01 | `Result := 1;` | Delphi | FP2003 "Unexpected token `Result` at start of statement" | poor | "Delphi `Result :=` – FPAS returns with `return Value;`" | parser |
| H-CALL-02 | `F := 1;` inside function `F` | Pascal | FP3006 "Type mismatch in assignment: expected `function(): integer`" | poor | "Assigning to the function name does not set its result; write `return Value;`" | sema |
| H-CALL-03 | `Exit(1);` | Delphi/FPC | FP3003 "Unknown procedure `Exit`" | poor | "Write `return 1;` (or `return;` in a procedure)" | sema |
| H-CALL-04 | `F(X => 1)` | Ada | FP2005 "Expected expression, found `>`", +5 | poor | "Ada named argument – FPAS writes `F(X := 1)`" | parser |
| H-CALL-05 | `Inc(A)` for a `var` parameter | Pascal | FP3027 "… its argument must be marked with `var`" | good | Keep | sema |
| H-CALL-06 | `@F` routine address | Pascal | FP1001 "Unexpected character `@`" | poor | "Use `F` as the callable value; no `@` is needed" | lexer/parser |
| H-CALL-07 | `TPoint.Create(1, 2)` | Delphi | FP3003 "Unknown function or procedure `TPoint.Create`" | poor | "Records have no constructors; write `TPoint(X := 1, Y := 2)`" | sema |
| H-CALL-08 | `Inc(X)` / `Dec(X)` built-ins | Pascal | FP3003 "Unknown procedure `Inc`" | poor | "Write `X := X + 1;`" | sema |
| H-CALL-09 | `WriteLn('x')` without `uses` | Pascal | FP3003 with hint to `uses Std.Console as Console;` | good | Keep | sema |
| H-CALL-10 | `Length(S)` | Pascal | FP3003 with hint `Value.Length(…)` and a stale remark about removed helper units | partial | Drop the stale remark; show `S.Length()` with the actual argument | sema |
| H-CALL-11 | `StrToInt(Text)` | Delphi | FP3003 with hint to `Parse.Int(Text).Unwrap()` | good | Keep | sema |
| H-CALL-12 | `IntToStr(3)` | Delphi | FP3003 with generic hint to `uses Std.Conv as Conv;` and `Conv.Name` | partial | Name the member: `Conv.IntToStr(3)` | sema |

## Statements and control flow

| ID | Habit | Origin | Today | Rating | Proposed | Stage |
| --- | --- | --- | --- | --- | --- | --- |
| H-STMT-01 | `procedure … end;` | Pascal | FP2001 "Expected `end procedure;`, found `end;`" | good | Keep | parser |
| H-STMT-02 | `if … then begin … end;` | Pascal | FP2001 "Expected `end if;`, found `end.`" at the program end | partial | Report at the inner `end;` and explain that `begin … end` after `then` is a nested block, not the branch body | parser |
| H-STMT-03 | `while … do begin … end;` | Pascal | FP2001 "Expected `end while;`, found `end.`" | partial | Same as H-STMT-02 | parser |
| H-STMT-04 | `if … then A else B;` without `;` before `else` | Pascal | FP2001 "Expected `;` after statement, found `else`", +1 | partial | "FPAS terminates every statement, also before `else`, and closes with `end if;`" | parser |
| H-STMT-05 | `case X of 1: …` | Pascal | FP2001 "Expected `when` at the start of a case arm", +1 | good | Keep; drop the `end case;` follow-up when only arms were wrong | parser |
| H-STMT-06 | `for I in 1 .. 3 loop … end loop;` | Ada | FP2001 "Expected `:`, found `in`", +5 | poor | "FPAS writes `for I: integer := 1 to 3 do … end for;`" | parser |
| H-STMT-07 | `for I := 1 to 3 do` with outer `I` | Pascal | FP2001 "Expected `:`, found `:=`", +3 | poor | "The loop declares its variable: `for I: integer := 1 to 3 do`" | parser |
| H-STMT-08 | `try … except … end;` | Delphi | FP2006 at `except`, +2 | poor | "FPAS has no exception handlers; return `Result of (T, E)` and use `try Expr` or `case`" | parser |
| H-STMT-09 | `X := nil` / `nil` | Pascal | FP3003 "Undefined identifier `nil`" | poor | "Absence is `None` of `Option of T`" | sema |

## Expressions and literals

| ID | Habit | Origin | Today | Rating | Proposed | Stage |
| --- | --- | --- | --- | --- | --- | --- |
| H-EXPR-01 | `6 and 3` on integers | Pascal | FP3006 with hint to `Bits.BitAnd` | good | Keep | sema |
| H-EXPR-02 | `1 /= 2` | Ada | FP2005 "Expected expression, found `=`", +1 | poor | "Ada `/=` – FPAS writes `<>`" | parser |
| H-EXPR-03 | `==`, `&&`, `\|\|`, `!=`, `!` | C-family | FP1001 for `&`, FP2005 for the second `=`, +6 | poor | One diagnostic per operator: "FPAS writes `=` / `and` / `or` / `<>` / `not`" | lexer/parser |
| H-EXPR-04 | `"abc"` double-quoted string | C-family | FP1001 "Unexpected character `\"`", +1 | poor | "Strings use single quotes: `'abc'`", one diagnostic for the whole literal | lexer |
| H-EXPR-05 | `S[1]` as first character | Pascal | not detectable statically | deferred | Covered by docs; a runtime index error already names the index | – |

## Comments

| ID | Habit | Origin | Today | Rating | Proposed | Stage |
| --- | --- | --- | --- | --- | --- | --- |
| H-LEX-01 | `{ comment }`, `(* comment *)` | Pascal | FP1013 "`{...}` is not valid comment syntax" | good | Keep | lexer |
| H-LEX-02 | `-- comment` | Ada | FP2001 "Expected `begin`, found `-`", +3 | poor | "Ada comment – FPAS comments start with `//`" | lexer/parser |

## Candidates not yet probed

To be probed in the first experiment run: `with … do`, `SetLength`, `High`/`Low`, `Ord`/`Chr`
as free functions, `string[N]`, `array[1..N] of T`, `record` with `case` variant part,
`out`/`const` parameters, `procedure of object`, `Ada package … is`, `declare … begin`,
`exit when`, `raise`, `inherited`, `property`, `set of`,
`Format('%d', [X])`, `Assigned(X)`, `Halt`.
