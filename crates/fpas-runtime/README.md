# FPAS runtime

`fpas-runtime` keeps VM execution, runtime values, memory ownership, native
standard-library operations, and host integration in one crate. Portable code
types come from `fpas-code` and are reexported for execution clients. Shared
standard-library declarations come from `fpas-builtins`.

The runtime accepts verified code, prepares aggregate layouts and shared string
constants, then executes it through `execution::Vm`. Values and mutable capture
cells follow the [ownership contract](../../docs/pascal/program-structure/runtime-ownership.md).
Host adapters connect FPAS operations to independently owned ACP/MCP transports
and other native services.

Run runtime checks with:

```sh
cargo test -p fpas-runtime
```

VM, value, and standard-library unit tests run in the ordinary library target.
Integration targets cover process-terminal access, server lifetime, and
[process-isolated memory checks](tests/gc_memory/README.md).

See the [workspace architecture](../../docs/rust-architecture.md).
