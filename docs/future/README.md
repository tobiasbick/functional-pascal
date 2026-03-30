# Future Features

Features planned for later versions of Functional Pascal. Implemented entries remain here only for roadmap tracking.

| Feature | Description |
|---------|-------------|
| ~~[Interfaces](interfaces.md)~~ | ~~Ad-hoc polymorphism, `interface` / `implements`, dynamic dispatch~~ — **implemented** (`interface`/`extends`/`implements`, nominal typing, `CallVirtual` dispatch) |
| ~~[Reference Types](references.md)~~ | ~~`ref` type constructor for shared, heap-allocated values~~ — **implemented** (`ref`, `new ... with ... end`, implicit dereference) |
| ~~[Record Extensions](record-extensions.md)~~ | ~~Default field values, `with` update expression, recursive record types~~ — **implemented** (default field values, `base with Field := Value; … end` update expression) |
| ~~[Stdlib Extensions](stdlib-extensions.md)~~ | ~~String padding/repeat, timer events, `for-in` over dict, index access~~ — **fully implemented** (`Dict.Map`/`Filter`, `String S[I]`, `Console.ReadEventTimeout`/`PollEvent`, `for-in` over dict) |
| ~~[Compiler Directives](compiler-directives.md)~~ | ~~`{$IFDEF}`, `{$I}`, compiler switches~~ — **implemented** (`{$DEFINE}`/`{$UNDEF}`, `{$IFDEF}`/`{$IFNDEF}`/`{$ELSE}`/`{$ENDIF}`, `{$INCLUDE}` in project mode) |
| [Libraries](libraries.md) | Project kind `library`, export rules |
| [Generics Extensions](generics.md) | HKT, variance, default type arguments, specialization (implemented: constraints `Comparable`/`Numeric`/`Printable`, generic methods on records and interfaces) |
| ~~[Error Handling Extensions](error-handling.md)~~ | ~~Chaining combinators~~ — **implemented** (`Map`, `AndThen`, `OrElse`) |
| ~~[Parallel VM Execution](parallel-vm.md)~~ | ~~Automatic multi-core `go` tasks via thread pool~~ — **implemented** (thread pool, crossbeam channels, `Worker`/`SharedState` architecture) |
