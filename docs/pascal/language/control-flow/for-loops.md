# For loops

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`for_stmt`).

## Counting up

```pascal
for I: integer := 1 to 10 do
begin
  WriteLn(I);
end;
```

## Counting down

```pascal
for I: integer := 10 downto 1 do
begin
  WriteLn(I);
end;
```

## Implementation (contributors)

Counting loops include both bounds. An empty ascending or descending range executes
no iterations. After the last iteration, including one ending with `continue`, the
loop exits before incrementing or decrementing the counter. This also applies when
the terminal bound is the minimum or maximum integer value.

Counting `for` lowers to explicit IR blocks and typed integer register instructions.
The initial comparison rejects empty ranges; the loop back edge checks equality
with the saved end bound before advancing the counter. `for-in` uses a saved
collection and an integer index checked against its length.

| Concern | Location |
|---------|----------|
| Lowering | [`control_flow.rs`](../../../../crates/fpas-compiler/src/lowering/control_flow.rs) |
| Counting loops | [`counting.rs`](../../../../crates/fpas-compiler/src/lowering/control_flow/counting.rs) |
| Opcodes | [`instruction.rs`](../../../../crates/fpas-bytecode/src/instruction.rs) |
| VM | [`dispatch.rs`](../../../../crates/fpas-vm/src/vm/dispatch.rs) |

## See also

- [For-in](for-in.md)
- [Break and continue](break-continue.md)
