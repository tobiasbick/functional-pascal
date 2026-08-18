# UMB-90 verification matrix

Use `pending`, `pass`, `rejected`, `blocked`, or `not-applicable`. A rejected or
blocked row needs evidence in [progress.md](progress.md); it is not a pass.

| ID | Requirement | Evidence | Status |
|---|---|---|---|
| `V90-01` | Compatibility names accepted and rejected live-image updates | Classification tests or child-not-started evidence | pending |
| `V90-02` | Incompatible updates are rejected before the live image changes | Atomic reject tests or child-not-started evidence | pending |
| `V90-03` | A recoverable old image remains until the new image commits | Rollback tests or child-not-started evidence | pending |
| `V90-04` | Memory, disk, snapshot, and retention bounds hold | Limit tests or child-not-started evidence | pending |
| `V90-05` | Recording capture is not used as a live-image snapshot store | Inventory plus later negatives | pending |
| `V90-06` | JSONL and DAP expose equivalent reload success and failures | Paired adapter tests | pending |
| `V90-07` | VS Code exposes only supported reload UX | Extension-host tests | pending |
| `V90-08` | Current docs describe implemented behavior and limitations only | Documentation review | pending |
| `V90-09` | No FPAS syntax, semantic, or language-spec change | Diff review | pending |
| `V90-10` | Formatting, locked workspace build/tests, changed-library Clippy, extension tests, and diff check pass | Commands recorded in progress | pending |

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
