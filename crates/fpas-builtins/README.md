# Standard-library declarations

`fpas-builtins` owns the shared names and shapes of native FPAS standard-library
declarations. Semantic analysis uses this catalog to register types and routines;
the compiler uses it to retain layouts; the runtime uses the same declarations
when constructing native values.

The catalog contains no runtime values, host handles, filesystem access, or
interpreter state. Native operations and their value builders belong to
`fpas-runtime`. Console variant order and native error payload order form part of
the language contract and must agree on both sides of this boundary.

Its only crate dependency is `fpas-acp-model`, which supplies generated ACP
declarations without connection drivers. Run catalog tests with
`cargo test -p fpas-builtins`.

See the [workspace architecture](../../docs/rust-architecture.md).
