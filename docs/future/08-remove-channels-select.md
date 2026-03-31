# Remove: Channels and Select

> Priority: 8 — after core simplifications, trim concurrency surface.

## What to remove

- `channel of T` type constructor.
- `select` statement with `case ... from` arms.
- `default` arm in `select`.
- `Std.Channel` unit (`Make`, `MakeBuffered`, `Send`, `Receive`,
  `TryReceive`, `Close`).
- Keywords: `channel`, `select`, `default`, `from`.

## What stays

- `go` keyword for spawning tasks.
- `task` type.
- `Std.Task` unit: `Wait`, `WaitAll`.

The fork-join pattern (`go` + `Wait`) covers parallel computation. The
Mandelbrot example demonstrates this: spawn one task per row, wait for all
results in order. No channels or select needed.

## Scope

- **Lexer/token:** remove `channel`, `select`, `default`, `from` keywords.
- **Parser:** remove `channel of T` from `type_expr`, remove `select_stmt`
  and `select_arm` productions.
- **Sema:** remove channel type checking, select validation.
- **Compiler:** remove channel/select bytecode generation.
- **Bytecode/VM:** remove channel opcodes and select implementation.
- **Std registry:** remove `Std.Channel` unit registration.
- **fpas-std:** remove or disable channel runtime code.
- **Grammar (EBNF):** remove 4 keywords, `select_stmt`, `select_arm`,
  `channel` from `type_expr`.
- **Docs:** remove `08-concurrency.md` channel/select sections. Keep
  `go`/`task`/`Wait` sections.
