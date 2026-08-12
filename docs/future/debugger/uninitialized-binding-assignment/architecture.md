# Architecture

## Runtime state

`Worker` owns parallel register arrays:

- `registers: Vec<Value>` stores physical values;
- `register_initialized: Vec<bool>` distinguishes an initialized `unit` value
  from empty storage; and
- `active_register_count` bounds the live register prefix.

All VM register writes pass through `store_register`; consuming moves pass
through `take_register`. Frame release clears both the value and its bit. Task
suspension preserves both vectors together. Function parameters and captures
enter a frame initialized; other frame registers start empty.

Globals continue to use `Vec<Option<Value>>`, where `None` is empty storage.

## Debugger flow

1. A stop snapshot reads the explicit register bit or global `Option`.
2. Mutable empty local/global roots receive a `MutationTarget` with
   `initialized = false`.
3. Replacement evaluation runs once in the detached bounded evaluator.
4. Portable debug-type validation completes before live storage changes.
5. Atomic commit stores the complete root and invalidates stopped-state handles.
6. A later source initializer is an ordinary VM write and may overwrite the
   debugger-provided value.

Selectors require an existing root value. Empty fields, indexes, dictionary
entries, and enum or wrapper payloads therefore fail before evaluation or
commit.

## Protocol ownership

The Rust session is authoritative. JSONL maps `variable.set` and
`expression.set`; DAP maps `setVariable` and `setExpression`; VS Code forwards
the standard DAP requests. No protocol has a separate mutation engine.
