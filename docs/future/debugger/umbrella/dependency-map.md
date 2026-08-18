# Umbrella dependency map

## Primary order

```text
UMB-00 baseline and checkpoint
  |
  v
UMB-01 contract decomposition
  |\
  | +--> UMB-10 identity-bearing assignment
  | +--> UMB-20 function breakpoints and failure filters
  | +--> UMB-30 controlled lifecycle
  |              |
  |              v
  +----------> UMB-40 task quiescence and control
                 |\
                 | +--> UMB-50 interactive debuggee transport
                 |          |
                 |          v
                 |        UMB-60 attach and remote
                 |
                 +-----> UMB-70 data breakpoints and actions
                              |
                              v
                            UMB-80 record and replay
                              |
                              v
UMB-90 hot reload
      |
      +--> UMB-10B entered anonymous closures
      |
      v
    UMB-99 closure
```

`UMB-10`, `UMB-20`, and the contract-only portion of `UMB-30` may be analyzed
independently after `UMB-01`. Only one package is implemented at a time unless
their files, runtime invariants, and verification gates are proven disjoint.

## Dependency rationale

| Package | Requires | Reason |
|---|---|---|
| `UMB-00` | none | Establishes a trustworthy branch and verification baseline |
| `UMB-01` | `UMB-00` | Replaces broad deferred prose with fixed, testable contracts |
| `UMB-10` | `UMB-01` | Extends the existing mutation transaction and ownership rules |
| `UMB-20` | `UMB-01` | Function names and runtime failures already have stable metadata; no task-control mutation is required |
| `UMB-30` | `UMB-01` | Must define cleanup and rollback before changing frames or terminal task states |
| `UMB-40` | `UMB-30` contract | Task control must account for completion, unwind, waiters, and failure states |
| `UMB-50` | `UMB-40A` quiescence proof | Blocking host I/O must cooperate with pause, cancellation, and all-stop behavior |
| `UMB-60` | `UMB-50` | Attach and remote sessions reuse the separated authenticated transport and cleanup model |
| `UMB-70` | `UMB-40A` and stable identities | Data stops and actions require deterministic shared-state visibility and exact mutation identities |
| `UMB-80` | `UMB-40`, `UMB-50`, `UMB-70` identity hooks | Capture records scheduler and host events at stable observation points; unsupported effects are rejected |
| `UMB-90` | `UMB-80` versioned program identity | Hot reload needs a versioned live executable and compatibility proofs; the recording capture log is not a snapshot store |
| `UMB-10B` | `UMB-90` | A newly entered closure body needs a verified function identity in the versioned live executable |
| `UMB-99` | all resolved primary packages | Runs final parity, packaging, documentation, and backlog cleanup |

## Shared gates

Any package that changes portable debug metadata also depends on round-trip
coverage through `.fpascu`, linker relocation, and `.fpascp`. Any package that
changes a machine operation also depends on JSONL before DAP and VS Code.

If a prerequisite is rejected, dependent packages stop and are re-scoped in
this map before implementation continues.
