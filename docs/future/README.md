# Future Features

Open planning items for Functional Pascal. This directory is for ideas, rewrites, deferred work, and design notes that are not the current user-facing specification.

Current implemented behavior belongs under `docs/pascal/`, not here.

## Open planning items

| Area | Plan | Scope |
|------|------|-------|
| Debugging | [D2: Record, replay, and query](agent-wishes.md#d2-record-replay-and-query) | Unscheduled wish; scope and feasibility open; no implementation approved |
| Agent tools | [Source queries for coding agents](agent-source-queries.md) | Proposed name selection, bounded inspection, direct call relationships, and later checked refactorings |
| Rust development | [Test and build structure](test-build-structure.md) | Planned fixture repair, test grouping, profile selection, resource measurements and Windows/Linux acceptance; implementation not started |
| Language | [Strict ISO/IEC 14977 grammar](iso-14977-grammar.md) | Open decision on grammar authority, strict notation, and automated drift checks |
| FPAS 2 | [Language directions](fpas2-language-directions.md) | Recorded decisions, implemented topics and open proposals |
| Standard library | [Standard library roadmap](std-roadmap.md) | Byte-buffer design, future capabilities and links to their owning plans |
| UI | [Shared terminal and graphical UI](shared-ui.md) | Planned common UI and application migration during graphics work, after FPAS 2 |
| Networked applications | [Networked application platform](networked-applications/README.md) | Storage, security, concurrency, transports, interactive clients, operations, and distributed nodes |
| WebDAV | [WebDAV](webdav.md) | Deferred WebDAV client and server ideas |
| FTP | [FTP and FTPS](ftp.md) | Deferred FTP and FTPS ideas |

## Ideas and designs without an implementation plan

- [Hardware information](hardware-information.md): access to CPU parallelism, RAM, and related
  hardware information; recorded as an idea only.
- [Graphics and 3D rendering](graphics.md): application case for the shared FPAS 2
  foundations, with graphics-specific windowing, GPU, platform and deployment
  questions; no graphics implementation plan or API is agreed.
- [Rewrite in C](c-rewrite.md): the human overlord's intent to rewrite the implementation in pure
  C because of Rust build times and disk usage; the language stays unchanged; recorded as an idea
  only.

## Architecture records and development intake

| Area | Document | Scope |
|------|----------|-------|
| Planning | [AP workflow and templates](improve-syntax/README.md) | Package index, reusable decision/implementation work packages, dependencies and completion checks |
| Standard-library architecture | [Binding namespace specification](../specs/std-namespaces.md) | Permanent naming policy, current units and future allocations; availability remains documented in the handbook |
| Compiler | [Panic and language-limit follow-ups](compiler-panic-followups.md) | Intake for newly discovered compiler panics and language limitations |
| Dogfooding | [Dogfooding gaps](dogfooding-gaps.md) | Intake for gaps found while writing agent helper scripts in FPAS |

## Rules

- Keep planned or speculative behavior in `docs/future/`.
- Move behavior to `docs/pascal/` only after it is implemented.
- Keep each future plan updated with status, next steps, and verification notes when work starts.
- Record compiler panics and language limitations with source-level workarounds in [compiler-panic-followups.md](compiler-panic-followups.md).
- Remove completed planning notes once implemented docs and tests cover their scope.
- Keep permanent contributor policies under `docs/specs/`; their target
  allocations must distinguish planned names from implemented APIs.
