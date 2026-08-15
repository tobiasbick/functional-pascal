# Umbrella acceptance matrix

Rows remain `PENDING` until the package that owns them records current evidence.
Historical plan claims are not accepted as current evidence.

| ID | Contract | Applies to | Evidence required | Status |
|---|---|---|---|---|
| `UMB-G01` | No FPAS syntax, semantics, or language-spec change without explicit approval | all | Diff classification against `docs/pascal/language/` and compiler behavior | PENDING |
| `UMB-G02` | Runtime identities never depend on display text or adapter-local handles | `10`, `20`, `30`, `40`, `60`, `70`, `80`, `90` | Positive collision cases plus stale/foreign/ambiguous negatives | PENDING |
| `UMB-G03` | One Rust engine owns behavior | all behavior packages | VM/session tests and adapter code review show mapping only | PENDING |
| `UMB-G04` | JSONL is deterministic and automation-friendly | all exposed operations | Stable request/result/error transcripts, strict argument validation, bounded output | PENDING |
| `UMB-G05` | DAP behavior is equivalent and capability-aware | all exposed operations | DAP integration tests, negotiated capability and invalidation cases | PENDING |
| `UMB-G06` | VS Code exposes only implemented shared behavior | editor-facing packages | Extension-host success/failure/lifecycle tests and local VSIX smoke test | PENDING |
| `UMB-G07` | Task, frame, value, cell, and stop ownership is explicit | `10`, `30`, `40`, `60`, `70`, `80`, `90` | Same-owner positives; foreign/stale/escape negatives | PENDING |
| `UMB-G08` | State changes prepare and validate before one atomic commit | `10`, `30`, `40`, `70`, `90` | Failure preserves state/handles; success invalidates once | PENDING |
| `UMB-G09` | Execution, values, history, transport, and output are bounded | all | Limit, timeout, cancellation, overflow, and resource cleanup tests | PENDING |
| `UMB-G10` | Portable metadata survives object and program artifacts | metadata-changing packages | Compiler, bytecode, `.fpascu`, linker, and `.fpascp` round trips | PENDING |
| `UMB-G11` | Current docs describe only implemented behavior | all | `docs/pascal/tools/` review and link check | PENDING |
| `UMB-G12` | Focused Rust and protocol tests pass | each package | Package-specific commands recorded in `progress.md` | PENDING |
| `UMB-G13` | Repository format and build gates pass | each package | `cargo fmt --check`, `cargo build --workspace --locked`, `git diff --check` | PENDING |
| `UMB-G14` | Full workspace regressions are understood | package closure | `cargo test --workspace --locked --no-fail-fast`; every failure classified | PENDING |
| `UMB-G15` | Changed FPAS fixtures use canonical formatting | packages with `.fpas` files | Targeted `fpas fmt --check` or repository formatting script | PENDING |
| `UMB-G16` | Privacy-sensitive transport and logs reveal no host metadata by default | `50`, `60`, `80` | Redaction, path mapping, authentication, and bounded-log tests | PENDING |

## Package-specific minimum rows

| Package | Mandatory rows in addition to `G01`, `G03`, `G11`-`G14` |
|---|---|
| `UMB-10` | `G02`, `G04`-`G10` |
| `UMB-20` | `G02`, `G04`-`G06`, `G09`, `G10` when metadata changes |
| `UMB-30` | `G02`, `G04`-`G09` |
| `UMB-40` | `G02`, `G04`-`G09` |
| `UMB-50` | `G04`-`G09`, `G16` |
| `UMB-60` | `G02`, `G04`-`G10`, `G16` |
| `UMB-70` | `G02`, `G04`-`G10` |
| `UMB-80` | `G02`, `G04`-`G10`, `G16` |
| `UMB-90` | `G02`, `G04`-`G10`, `G16` when transported remotely |

`PASS` requires command evidence from the current package checkpoint. A known
unrelated failure is recorded as `BASELINE` with its exact target; it is never
silently counted as `PASS`.

