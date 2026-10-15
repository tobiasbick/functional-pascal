# Future Features

Open planning items for Functional Pascal. This directory is for ideas, rewrites, deferred work, and design notes that are not the current user-facing specification.

Current implemented behavior belongs under `docs/pascal/`, not here.

## Open planning items

| Area | Plan | Scope |
|------|------|-------|
| Language | [Strict ISO/IEC 14977 grammar](iso-14977-grammar.md) | Open decision on grammar authority, strict notation, and automated drift checks |
| Standard library | [Standard library roadmap](std-roadmap.md) | Future `Std.*` units and longer-term stdlib direction |
| Networked applications | [Networked application platform](networked-applications/README.md) | Storage, security, concurrency, transports, interactive clients, operations, and distributed nodes |
| WebDAV | [WebDAV](webdav.md) | Deferred WebDAV client and server ideas |
| FTP | [FTP and FTPS](ftp.md) | Deferred FTP and FTPS ideas |

## Ideas without an implementation plan

- [FPAS 2 language directions](fpas2-language-directions.md): retained design questions;
  no implementation is scheduled. Selected work uses the [AP workflow](improve-syntax/README.md).
- [Hardware information](hardware-information.md): access to CPU parallelism, RAM, and related
  hardware information; recorded as an idea only.
- [Graphics and 3D rendering](graphics.md): runtime, standard-library, build, and language gaps
  for windowing and GPU rendering; recorded as an idea only.
- [Rewrite in C](c-rewrite.md): the human overlord's intent to rewrite the implementation in pure
  C because of Rust build times and disk usage; the language stays unchanged; recorded as an idea
  only.
- [Agent wishes](agent-wishes.md): the agent's wish list — writing FPAS without training data,
  shared wins for humans and agents (tested documentation examples, richer diagnostics), and CLI
  tools with JSON output for debugging (replacing the JSONL protocol) and code navigation; DAP and
  LSP stay for humans. A1 and S1 are implemented; the
  remaining tooling wishes are ideas without an implementation plan.

## Architecture records and development intake

| Area | Document | Scope |
|------|----------|-------|
| Planning | [AP workflow and templates](improve-syntax/README.md) | Empty package index, reusable decision/implementation work packages, dependencies and completion checks |
| Compiler | [Panic and language-limit follow-ups](compiler-panic-followups.md) | Intake for newly discovered compiler panics and language limitations |
| Dogfooding | [Dogfooding gaps](dogfooding-gaps.md) | Intake for gaps found while writing agent helper scripts in FPAS |
| Documentation | [Example inventory](documentation-example-inventory.md) | Historical S1 baseline before the implemented Markdown example checker |

## Rules

- Keep planned or speculative behavior in `docs/future/`.
- Move behavior to `docs/pascal/` only after it is implemented.
- Keep each future plan updated with status, next steps, and verification notes when work starts.
- Record compiler panics and language limitations with source-level workarounds in [compiler-panic-followups.md](compiler-panic-followups.md).
- Remove completed planning notes once implemented docs and tests cover their scope.
