# VM bridge

Hosted Graph intrinsics use discriminants **331–342** (see [`graph.rs`](../../../../../crates/fpas-bytecode/src/intrinsic/graph.rs)). Core entrypoints:

| Pascal | Intrinsic |
|--------|-----------|
| `Application.Configure` | `ApplicationConfigure` (296) |
| `Application.Run` | `ApplicationRun` (331) |
| `Application.HostRequestQuit` | `HostRequestQuit` (332) |
| `Application.HostProcessNext` | `HostProcessNext` (335) |
| `Application.HostDispatchRedraw` | `HostDispatchRedraw` (337) |
| `Application.OpenForTest` | `OpenForTest` (379) |
| `Application.TestSendKey` | `TestSendKey` (380) |

Shared internal event normalization lives in [`fpas-std/src/ui/`](../../../../../crates/fpas-std/src/ui/mod.rs) (`UiHost`, `UiEvent`).

Test intrinsics **379–380** are documented in [`test.md`](../../testing/test.md). Example: [`graph_smoke_test.fpas`](../../../../../tests/graph/graph_smoke_test.fpas).

## See also

- [`Std.Test`](../../testing/test.md)
- [Lifecycle](lifecycle.md)
- [Hosted dispatch overview](README.md)
