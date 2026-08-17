# UMB-70 verification matrix

Use `pending`, `pass`, `rejected`, `blocked`, or `not-applicable`. A rejected or
blocked row needs evidence in [progress.md](progress.md); it is not a pass.

| ID | Requirement | Evidence | Status |
|---|---|---|---|
| `V70-01` | Supported locations have exact identities and lifetimes | VM identity tests | pending |
| `V70-02` | Capture-cell destinations are accepted or remain rejected with evidence | Mutation tests plus `UMB-70A` decision | pending |
| `V70-03` | Data breakpoints stop deterministically on read, write, or change | VM/adapter tests or child-not-started evidence | pending |
| `V70-04` | Watchpoint overhead and retained state are bounded | Limit tests | pending |
| `V70-05` | Mutating breakpoint actions commit once and invalidate once | Transaction tests or child-not-started evidence | pending |
| `V70-06` | JSONL and DAP expose equivalent data-stop success and failures | Paired adapter tests | pass |
| `V70-07` | VS Code exposes only supported watchpoint UX | Extension-host tests | pending |
| `V70-08` | Current docs describe implemented behavior and limitations only | Documentation review | pass |
| `V70-09` | No FPAS syntax, semantic, or language-spec change | Diff review | pass |
| `V70-10` | Formatting, locked workspace build/tests, changed-library Clippy, extension tests, and diff check pass | Commands recorded in progress | pass |

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
