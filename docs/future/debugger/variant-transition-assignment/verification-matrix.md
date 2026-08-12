# Verification matrix

Status values are `PLANNED`, `PASS`, or `BLOCKED`.

| ID | Acceptance case | Planned evidence | Status |
|---|---|---|---|
| VTA-T01 | Inactive single-field enum variant is selected by exact qualified target | VM session cases | PASS |
| VTA-T02 | `Option.None` becomes `Option.Some` through `Some.value` | VM and source fixture | PASS |
| VTA-T03 | `Result.Ok` and `Result.Error` switch in both directions | VM and source fixture | PASS |
| VTA-T04 | Active qualified target behaves like current active payload editing | VM regression beside existing payload tests | PASS |
| VTA-T05 | Nested record, array, and dictionary prefixes replace only the selected wrapper | VM and source-fixture transitions from inactive variants | PASS |
| VTA-T06 | Mutable local, global, parameter, and capture roots follow existing ownership | VM session cases | PASS |
| VTA-T07 | Selected child task owns evaluation and commit | JSONL task transcript | PASS |
| VTA-T08 | Replacement expression evaluates once under shared limits | call counter, cancellation, and operation-limit cases | PASS |
| VTA-T09 | Wrong payload type preserves the old complete variant | VM, JSONL, and DAP negative cases | PASS |
| VTA-T10 | Unknown or ambiguous variant and payload names are actionable | parser/session/protocol negative cases and variant/field collision precedence | PASS |
| VTA-T11 | Unqualified inactive field is rejected without variant guessing | VM and protocol negative cases | PASS |
| VTA-T12 | Fieldless and multi-field variants reject descendant construction and point to root constructors | VM and protocol negative cases | PASS |
| VTA-T13 | Uninitialized, immutable, hidden, stale, and foreign targets remain rejected | VM and protocol negative cases | PASS |
| VTA-T14 | Failed DAP requests emit no invalidation; success emits one negotiated invalidation | DAP transcript | PASS |
| VTA-T15 | JSONL and DAP error codes and resulting values are equivalent | paired transcripts | PASS |
| VTA-T16 | VS Code debug sessions forward qualified `setExpression` and refresh Variables | Extension Host custom-request test | PASS |
| VTA-T17 | Continuing execution observes the new target variant | source fixture output | PASS |
| VTA-T18 | Existing payload mutation and complete variant replacement do not regress | focused existing suites | PASS |
| VTA-T19 | FPAS fixture formatting is stable | `fpas fmt --check tests/debugger/fixtures/variant_transition.fpas` | PASS |
| VTA-T20 | Rust formatting and build pass | `cargo fmt --check`; `cargo build` | PASS |
| VTA-T21 | Full workspace tests pass | `cargo test --workspace --no-fail-fast` | PASS |

## Required protocol assertions

For every successful protocol mutation, assert the rendered result, refreshed
aggregate shape, continuation output, handle expiry, and task identity. For
every rejected mutation, assert the stable code and hint, unchanged value,
preserved current handles, stopped state, and absence of DAP invalidation.
