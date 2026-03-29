# Future Features

Features planned for later versions of Functional Pascal, not yet implemented.

| Feature | Description |
|---------|-------------|
| [Interfaces](interfaces.md) | Ad-hoc polymorphism, `interface` / `implements`, dynamic dispatch |
| [Reference Types](references.md) | `ref` type constructor for shared, heap-allocated values |
| [Record Extensions](record-extensions.md) | Default field values, `with` update expression, recursive record types |
| [Stdlib Extensions](stdlib-extensions.md) | String padding/repeat, timer events, `for-in` over dict, index access |
| [Compiler Directives](compiler-directives.md) | `{$IFDEF}`, `{$I}`, compiler switches |
| [Libraries](libraries.md) | Project kind `library`, export rules |
| [Generics Extensions](generics.md) | HKT, variance, specialization (constraints **implemented**: `Comparable`, `Numeric`, `Printable`) |
| ~~[Error Handling Extensions](error-handling.md)~~ | ~~Chaining combinators~~ — **implemented** (`Map`, `AndThen`, `OrElse`) |
| ~~[Parallel VM Execution](parallel-vm.md)~~ | ~~Automatic multi-core `go` tasks via thread pool~~ — **implemented** (thread pool, crossbeam channels, `Worker`/`SharedState` architecture) |
