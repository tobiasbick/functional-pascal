# AP55.2: Implement and adopt resource cleanup

Package: [AP55](README.md). Status: complete.

The approved [contract](contract.md) is implemented across these owners:

- `crates/fpas-lexer`, `crates/fpas-parser`, `crates/fpas-sema` and
  `crates/fpas-fmt` handle resource locals, contextual release members,
  resource escape checks and formatting. Generic and imported record types
  retain their release capability; synchronous borrows remain available.
- `crates/fpas-compiler`, `crates/fpas-ir`, `crates/fpas-code` and
  `crates/fpas-runtime` lower and execute lexical cleanup, including the
  acquisition handoff, task joins, cancellation masking, postconditions and
  failure precedence. The native catalogue preserves explicit close behavior
  and retires resources before their owning block completes.
- `crates/fpas-code`, `crates/fpas-unit`, `crates/fpas-program` and
  `crates/fpas-linker` validate and transport the new instructions and release
  metadata. The bytecode ABI is version 32 and the unit format is version 25.
- `crates/fpas-language-service`, `crates/fpas-lsp`, `editors/vscode`,
  `docs/specs/grammar.ebnf` and `docs/pascal` cover editor queries,
  highlighting, source syntax and the current language reference.
- `crates/fpas-debug`, `crates/fpas-cli` and `crates/fpas-runner` preserve
  cleanup obligations and report supplemental failures through the debugger
  and diagnostic transports. Frame replacement cannot discard live cleanup.

The regression suite covers reverse release order, loop exits, failed
acquisition, cancellation at the producer handoff, task joining and suspended
cleanup. It checks postconditions after release, anonymous and generic routines,
imported private release members, safe borrowing and rejected escapes. Nested
release failures stop the failing disposer without abandoning outer cleanup;
Result errors and None retain their original outcome with supplemental
diagnostics. Real file, process, socket, ACP, MCP and HTTP cases cover native
release and already-closed aliases. Formatter, artifact, editor and debugger
tests cover their respective transport and user-facing contracts.
