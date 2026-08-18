# Source-debugger roadmap

There is no active source-debugger implementation plan. The completion
umbrella has been resolved and removed.

Implemented behavior, supported limits, and explicit rejections are documented
under [`docs/pascal/tools/`](../../pascal/tools/debugger.md). The debugger is a
single FPAS source-level engine shared by JSONL and DAP/VS Code; it does not
provide native Rust/VM process debugging.

[deferred.md](deferred.md) is the sole backlog for independently useful future
debugger capabilities. It currently contains no open package.

Future work must add one bounded, independently testable row there before
creating another resumable plan. Do not restore completed umbrella history as
backlog.
