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

Counting `for` lowers to fused local opcodes (`IncLocal` / `DecLocal`, `JumpIfLocalGt` /
`JumpIfLocalLt`) so hot loops avoid stack traffic for the counter and bound test. `for-in`
uses `IncLocal` / `SetLocalPop` for the index and element store; the bound test stays as
`LtInt` + `JumpIfFalse`.

| Concern | Location |
|---------|----------|
| Lowering | [`for_loops.rs`](../../../../crates/fpas-compiler/src/compiler/stmt/loops/for_loops.rs) |
| Opcodes | [`op.rs`](../../../../crates/fpas-bytecode/src/op.rs) |
| VM | [`stack_scope.rs`](../../../../crates/fpas-vm/src/vm/execute/stack_scope.rs), [`control_calls.rs`](../../../../crates/fpas-vm/src/vm/execute/control_calls.rs) |

## See also

- [For-in](for-in.md)
- [Break and continue](break-continue.md)
