# UMB-60 verification matrix

Use `pending`, `pass`, `rejected`, `blocked`, or `not-applicable`. A rejected or
blocked row needs evidence in [progress.md](progress.md); it is not a pass.

| ID | Requirement | Evidence | Status |
|---|---|---|---|
| `V60-01` | Launch-owned sessions keep `attach` false until a proven subset exists | Capability and negative request tests | pending |
| `V60-02` | Local attach discovery is authorized and does not leak host paths | VM/adapter tests or child-not-started evidence | pending |
| `V60-03` | Attach disconnect ownership is explicit and does not hide execution | VM/host tests or child-not-started evidence | pending |
| `V60-04` | Attached sources map through the portable identity contract | Metadata/source tests or child-not-started evidence | pending |
| `V60-05` | Remote sessions authenticate, negotiate versions, and recover | Adapter tests or child-not-started evidence | pending |
| `V60-06` | Native debugging is accepted only without a second semantic engine | `UMB-60C` decision | pending |
| `V60-07` | JSONL and DAP expose equivalent attach success and failures | Paired adapter tests | pending |
| `V60-08` | VS Code exposes only supported attach UX | Extension-host tests | pending |
| `V60-09` | Current docs describe implemented behavior and limitations only | Documentation review | pending |
| `V60-10` | No FPAS syntax, semantic, or language-spec change | Diff review | pending |
| `V60-11` | Formatting, locked workspace build/tests, changed-library Clippy, extension tests, and diff check pass | Commands recorded in progress | pending |

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
