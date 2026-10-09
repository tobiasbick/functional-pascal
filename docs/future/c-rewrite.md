# Rewrite in C

> **⚠️ Disclaimer:** The human overlord has spoken. Everyone else is rewriting their programs in
> Rust; this hobby project intends to go the other way and rewrite its implementation in pure C.
> This is a statement of intent, not a plan: no schedule, no design, no promise. If you depend on
> this project, please reconsider your life choices — the human sincerely hopes nobody does.

## Status

Idea only. No implementation plan. Recorded at the human's request; the Rust implementation remains
the current one until something replaces it.

## Motivation

Not the syntax. The toolchain:

- **Build times.** A full Rust build and test run of the workspace takes far too long for a hobby
  project.
- **Disk usage.** The Cargo `target/` directory of this repository measured 57 GB at the time of
  writing.
- **The agent writes most of the code anyway.** With LLMs getting better at writing code, the
  safety net of the Rust compiler matters less to the human than build speed and a small, simple
  toolchain.

## Scope

- **Implementation only.** Compiler, VM, standard-library runtime, CLI, language server, and debugger
  adapter would be rewritten. The FPAS language itself — syntax, semantics, and the specification
  under `docs/pascal/` — stays as it is.
- **The FPAS test suite is the contract.** The `*_test.fpas` programs and golden files under
  `tests/` are independent of Rust and would decide whether the C implementation behaves like the
  current one. The Rust unit tests (roughly 110k of about 258k lines of Rust) would not carry over.

## Open questions

- **C standard and toolchain.** Which standard (C11, C17, C23) and which compilers on Windows and
  Linux. Which build system — ideally one that needs nothing beyond a C compiler.
- **Memory safety.** The Rust workspace forbids `unsafe` code; in C that guarantee moves from the
  compiler to discipline and tooling. Sanitizers (ASan, UBSan) and fuzzing of lexer, parser,
  bytecode verifier, and `.fpascu` loading would need to be part of the normal test run.
- **Dependencies.** The Rust implementation uses crates for TLS (`rustls`), terminal control
  (`crossterm`), JSON, TOML, glob patterns, hashing, and Unicode segmentation. TLS and cryptography
  must not be self-written; a C library such as mbedTLS or OpenSSL would be needed. The rest is a
  choice between small libraries and own code.
- **Migration path.** Big-bang rewrite, or component by component behind the existing file formats
  (bytecode, `.fpascu` units, program images) so the C and Rust parts can be checked against each
  other.
- **Editor tooling.** The VS Code extension talks to `fpas lsp` and the DAP adapter, so it would keep
  working as long as the C implementation speaks the same protocols.
