# Future Features

Open planning items for Functional Pascal. This directory is for ideas, rewrites, deferred work, and design notes that are not the current user-facing specification.

Current implemented behavior belongs under `docs/pascal/`, not here.

## Open planning items

| Area | Plan | Scope |
|------|------|-------|
| Language | [Strict ISO/IEC 14977 grammar](iso-14977-grammar.md) | Open decision on grammar authority, strict notation, and automated drift checks |
| Language | [Syntax improvement implementation plan](improve-syntax/README.md) | Pascal-family syntax changes, explicit contracts, structured task scopes; one directory per package (AP01–AP29) with individually verifiable work packages, dependencies, and regression checks |
| Standard library | [Standard library roadmap](std-roadmap.md) | Future `Std.*` units and longer-term stdlib direction |
| Networked applications | [Networked application platform](networked-applications/README.md) | Storage, security, concurrency, transports, interactive clients, operations, and distributed nodes |
| WebDAV | [WebDAV](webdav.md) | Deferred WebDAV client and server ideas |
| FTP | [FTP and FTPS](ftp.md) | Deferred FTP and FTPS ideas |

## Ideas without an implementation plan

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
  LSP stay for humans; the human is still undecided; recorded as an idea only.

## Architecture records and development intake

| Area | Document | Scope |
|------|----------|-------|
| Compiler | [Panic and language-limit follow-ups](compiler-panic-followups.md) | Intake for newly discovered compiler panics and language limitations |
| Dogfooding | [Dogfooding gaps](dogfooding-gaps.md) | Intake for gaps found while writing agent helper scripts in FPAS |

## Rules

- Keep planned or speculative behavior in `docs/future/`.
- Move behavior to `docs/pascal/` only after it is implemented.
- Keep each future plan updated with status, next steps, and verification notes when work starts.
- Record compiler panics and language limitations with source-level workarounds in [compiler-panic-followups.md](compiler-panic-followups.md).
- Remove completed planning notes once implemented docs and tests cover their scope.
