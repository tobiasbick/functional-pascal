# AP47.7: Final acceptance, documentation and performance

Status: Planned; implementation not authorized. Depends on AP47.6.
Completion: [AP47](README.md#dependencies-and-work-packages).
This file names future checks; none are required to run for the planning change.

## Deliverable

Verify the integrated selected contract, finish publication-quality examples and
migration guidance, and demonstrate that automatic errors do not reintroduce
per-instruction VM overhead. Earlier packages own their functional regressions
and consumer migrations; this package closes cross-package gaps rather than
postponing a broken checkout or missing handbook work until the end.

## Final acceptance scenarios and proof owners

| Observable contract | Primary proof owner | Distinct integration risk |
| --- | --- | --- |
| Declare/construct/match native and generic errors; reject invalid syntax, Cause and constraints | Existing parser/sema/compiler language fixtures | Formatter recovery and unsaved-source LSP/CLI queries |
| Message(), Console output and cause order agree; payloads stay typed | Runtime value/display plus a complete source program | Two generic error instances, aliases and construction through an imported generic helper/callable retain concrete identity in a stripped image |
| Constructor callable inference and MapError count/order | Existing Result/callable source owner | Scheduled suspension, cancellation and trace/failure-capture lifecycle |
| Host and parser conditions choose stable cases and preserve native/parser data | Production Std owner fixtures | Real filesystem/process permissions, backend metadata and public generated signatures |
| Task/network outcomes and selected callbacks preserve semantics | Channel/selection/network fixtures | Ownership after timeout, supervision, TLS, server shutdown budgeting and Utf8.Decode versus Sse.Finish error ownership |
| Layered applications retain original causes and handle local configuration validation separately | One complete ConfigError example | Source HTTP/field-helper mappings and cached libraries |
| Unchanged success paths and unrelated workloads retain performance | Production benchmark harness | Release build selection, repeated medians and no concurrent load |

Reuse existing owner proof. Add no test-only production entry point, source-grep
test, duplicated inventory assertion or copied implementation as an expected
result. Coverage may span multiple layers only when transport or lifecycle risk
differs. Negative fixtures must fail for their intended static/runtime guard.
Record unavailable pre-fix proof honestly rather than fabricate it.

## Complete user example

Keep one small source program under examples/pascal/errors and a compact-reference
fence using ordinary FPAS syntax. It declares ConfigError with FileFailure,
MissingSetting and InvalidSetting, maps a real FsError with the constructor value,
matches the nested cause, and prints it directly. An ordinary closure shows how
to add operation/path context when a wrapper has more than one parameter.

The same example shows a generic Error-constrained wrapper and reports a local
validation error without inventing a file cause. Exercise that wrapper with two
cause families so the example cannot pass by sharing one erased error layout.
The existing reference/example test must compile/run the fence. All examples distinguish selected future rules
from implementation until the corresponding package ships. Do not introduce a
second error tutorial API, generated reporter function or hidden wrapper.

## Documentation and migration closure

- Align the real grammar, type/error-handling/constraints/callable handbooks,
  native operation catalogs, compact reference and Std pages with shipped code.
- Regenerate intrinsic APIs and check reproducibility, including error-family
  declarations, message labels and nested selection Outcome types.
- Check all affected `.fpas` files and Rust-embedded programs, project templates,
  editor fixtures/snippets, performance generators and repository authoring
  skills. Change skills only if they encode obsolete implemented error rules.
- Inventory every selected public interface and alias from standard-errors.md;
  no selected string-error or duplicate wrapper form remains. Out-of-scope
  application/SDK/TUI boundaries keep their deliberate contracts through explicit
  mappings. Do not sweep unrelated strings simply because they contain Error.
- Owned precompiled libraries/caches use the new artifact format. Rejection of
  unsupported standalone images explains rebuild requirements. Inspect failure
  capture, debugger value views and live-image compatibility with typed causes.
- Update the future direction/gap assessment to describe actual remaining gaps
  only as implementation is delivered. Keep plan completion separate from Git
  publication and mark central completion only after every package is checked.

## Performance plan and gates

Before AP47.2 changes measured code, use cargo bench-fpas to save the unchanged
full suite and VM group. Keep compiler/toolchain, power settings and machine load
comparable. The harness owns rebuilding the release CLI. Capture three serial
runs for medians, with correctness checks and no builds/tests running alongside
timings. Re-run VM microbenchmarks after metadata, language and callback changes;
run the complete suite after Std integration. Investigate any repeatable runtime
slowdown; the recent trace-dispatch regression is specifically not an acceptable
tradeoff for typed errors.

For unchanged VM rows with baseline medians of at least 20 ms, a repeatable
increase over 5% blocks completion until repaired or the user explicitly accepts
the measured tradeoff. Smaller rows are reported with raw samples rather than
percentage gates. Compare group sums of row medians as such, not as whole-suite
wall time. Codec/tooling/host changes get their own complete before/after report;
no material regression is hidden by a geometric mean or an improvement elsewhere.

After AP47.6's original full-suite comparison, register a bounded `errors` group
before further feature performance tuning. AP47.4 makes its language available;
suite registration waits so the original full-suite IDs stay comparable.
Planned workloads:

| Workload | Boundary and correctness | Control |
| --- | --- | --- |
| Error construction and wrapping, 100000 iterations after warmup | Construct typed payloads without rendering; independently known variant/payload checksum | Equivalent ordinary enum/Result construction in the same build |
| Generic error construction through a helper/callable, 100000 iterations | Alternate two concrete cause families; check owner/Cause identity and reuse of factories | Direct concrete construction in the same build; report shared metadata/factory size and hidden-call overhead |
| MapError Ok, 1000000 iterations | Mapper counter stays zero; success checksum is exact | Equivalent explicit case, matched inputs |
| MapError Error, 1000000 iterations | Mapper count equals iterations and transformed payload checksum is exact | Explicit case and constructor callable versus ordinary function |
| Cause output, 1000 renders of depth 16 | Compare one independently specified small golden and checked output size/checksum for the repeated workload | Direct Message() and Console host-free renderer use the same production renderer |
| Deep cause traversal and artifact round trip | One depth-512 chain renders/drops safely and retains its terminal typed cause after decode | Structural correctness, not a fragile wall-clock assertion |

Do not claim an old-checkout baseline for syntax unavailable on that checkout.
New feature rows use matched in-build controls and then a fresh same-suite
baseline for subsequent tuning. Keep IDs/arguments/selection fixed for a
comparison, never fabricate zero/missing rows. Record settled measurements in
docs/bench/history.md with workload boundaries and any changed suite baseline.

Future command sequence, using the groups available at each stage:

```text
cargo bench-fpas save ap47-before
cargo bench-fpas save ap47-vm-before --group vm
cargo bench-fpas compare ap47-vm-before --group vm
cargo bench-fpas compare ap47-before
cargo bench-fpas save ap47-errors --group errors
cargo bench-fpas compare ap47-errors --group errors
cargo bench-fpas record "native typed operational errors"
```

The `errors` commands are valid only after its workloads have been registered.
Register them after the original full-suite comparison, then save a fresh
full-suite baseline including the new rows for subsequent tuning.
Do not weaken the harness's group/row validation to compare different suites.

## Required integration checks

Future delivery runs the applicable per-package checks and then:

```text
cargo fmt --all -- --check
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo run -p fpas-sema --example export_intrinsic_std_api
fpas fmt --check lib apps examples tests
fpas test tests/suite.fpasprj
node editors/vscode/scripts/verify-grammar.mjs
git diff --check
```

Use the newly built distribution for fpas commands. Check affected app/example
projects with their actual manifests, including unsaved-source and source-query
fixtures. Run native socket/TLS/process fixtures with permitted local access;
reproduce sandbox-only failures outside restrictions before changing product
behavior. The source error grammar must pass existing LSP and editor tests.
Relative links, executable documentation examples and reproducible generated API
output complete the documentation check. Report exact checks and measurements
in the delivery, not chronological logs or machine metadata inside the plan.
