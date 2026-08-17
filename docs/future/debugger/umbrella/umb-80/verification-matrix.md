# UMB-80 verification matrix

Use `pending`, `pass`, `rejected`, `blocked`, or `not-applicable`. A rejected or
blocked row needs evidence in [progress.md](progress.md); it is not a pass.

| ID | Requirement | Evidence | Status |
|---|---|---|---|
| `V80-01` | Recording envelope names program identity without host paths | Envelope tests or child-not-started evidence | pending |
| `V80-02` | Scheduler and supported host events are captured only at explicit boundaries | Capture tests or child-not-started evidence | pending |
| `V80-03` | Memory, disk, event, snapshot, and retention bounds hold | Limit tests or child-not-started evidence | pending |
| `V80-04` | Unsupported nondeterminism is rejected before replay is claimed | Rejection tests or child-not-started evidence | pending |
| `V80-05` | Recording-disabled execution matches the current launch-owned path | Recording-off tests or child-not-started evidence | pending |
| `V80-06` | JSONL and DAP expose equivalent record/replay success and failures | Paired adapter tests | pending |
| `V80-07` | VS Code exposes only supported recording UX | Extension-host tests | pending |
| `V80-08` | Current docs describe implemented behavior and limitations only | Documentation review | pending |
| `V80-09` | No FPAS syntax, semantic, or language-spec change | Diff review | pending |
| `V80-10` | Formatting, locked workspace build/tests, changed-library Clippy, extension tests, and diff check pass | Commands recorded in progress | pending |

## Standard final commands

```powershell
cargo fmt --all -- --check
cargo build --workspace --locked
cargo clippy -p fpas-vm -p fpas-debug --lib -- -D warnings
cargo test --workspace --locked --no-fail-fast
npm test --prefix editors/vscode
git diff --check
```

Add targeted commands to [progress.md](progress.md) as ownership becomes
concrete. If `.fpas` fixtures change, also run their formatter check and the
applicable FPAS test bundle.
