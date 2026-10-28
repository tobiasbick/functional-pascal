# AP53 namespace migration verification

Status: verified. Package completion is tracked in
[AP53](../future/improve-syntax/ap53-std-namespaces/README.md).

## Scope and review baseline

The migration follows the binding [namespace specification](../specs/std-namespaces.md).
It preserves existing library behavior, moves public unit/type identities and
splits text formatting from numeric conversion. It adds no future buffer, UI,
network or multimedia features.

| Repository | Branch | Review baseline |
| --- | --- | --- |
| Functional Pascal | `codex/std-namespaces` | `aaf704dbba688269951c5945a2f5b9bdf1852e63` |
| fpas-into-the-dungeon | `codex/16-llm-integration` | `9b766b9db374ba2f7e0249d073f6011ae4f248ed` |

Both changes are reviewed against those fixed baselines. Delivery uses separate
co-authored commits and pushes; no merge is part of AP53.

## Regression boundaries

- All 28 intrinsic declarations and 10 public source exports have their canonical
  names. Private source units remain private; nonexistent parents do not import
  their children. Generated declarations retain their flat dotted filenames.
- Mixed-case aliases, imported nominal types, runtime diagnostics, formatter
  fixtures, CLI documentation and editor lookup follow the same names.
- The four formatting routines belong to `Std.Text.Format`; numeric conversion
  belongs to `Std.Math.IntToReal`. Fixed-signature routine values work through
  imported aliases, including named arguments and callbacks.
- Bytecode ABI 29 rejects old standalone program images before execution.
  Source-backed program and unit artifacts rebuild, then reuse their current
  identities. Envelope and object schema layouts do not change.
- Dungeon rules, generator fingerprints and protocol payloads are unchanged.
  Its compiler and standard library must come from the matching FPAS branch.

## Verification

Commands below are relative to the appropriate repository root. The FPAS
compiler is built with `--profile dev-fast`; no release, benchmark or dist build
is required by this change.

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed after review repairs |
| `cargo build --profile dev-fast` | Final ABI 29 build passed |
| `cargo test --workspace -j 8` | Passed, including program/bundle ABI goldens and callable-conversion regression |
| `fpas fmt --check lib apps examples tests tools` | Passed |
| `fpas test --std-lib lib --timeout 600 --jobs 4 tests/suite.fpasprj` | 590 passed, one intentional runner skip, zero failed |
| Explicit checks of all app/example manifests | 30 passed with the final ABI 29 compiler |
| Generated intrinsic API | 28 declarations; canonical ownership and stale-file removal verified |
| `fpas stdlib stage` with a disposable copy of the built library | 126 units compiled, zero reused; output sources, manifest and 28 API declarations identical to repository `lib` |
| Source artifact rebuild and reuse | Final ABI 29 build/reuse passed; old image rejected with FP4125 and rebuild guidance, without changing the old image |
| Imported `IntToReal` function value and typed filesystem errors | Both final compiler execution checks passed |
| `fpas doc Std.Text.Format` and `fpas doc Std.Math.IntToReal` with a scoped source context | Passed; declarations and handbook owners agree |
| Relative documentation link targets | 1,098 checked, none missing |
| Agent reference token budgets | 8,075 total and 922 in the entrypoint using `o200k_base` |
| Dungeon formatting and workspace check | Passed |
| Dungeon additional tool/fixture manifests | Six passed |
| `fpas test --timeout 600 --jobs 1 dungeon.fpasworkspace` with the matching compiler/library | Final isolated run: 183 passed, zero failed; earlier concurrent-run timeout documented below |
| `git diff --check` in both repositories | Passed |

Independent Spec and Standards agents reviewed the implementation. Confirmed
documentation, callable-lookup and artifact-identity findings are repaired.
Both final source reviews report no remaining confirmed findings. All required
final checks pass.

## Environment and coverage limits

Validation runs locally on Windows. Compiler and child-process paths select the
same FPAS build and source library. Network/process tests require ordinary local
loopback and temporary-directory access; restricted sandbox runs do not provide
evidence of their behavior.

The dungeon AI integration coverage uses scripted ACP peers and an actual
embedded MCP server. It does not claim a live external model/provider session.
Interactive applications are compiler-checked and covered by their scripted or
headless tests; no manual graphical/terminal session is claimed.

One dungeon run during parallel Rust testing passed 182 tests and did not observe
server readiness within the fixture's 30-second window. The unchanged focused test
passed. A bounded debugger run reached readiness in about 2.6 seconds with empty
stderr; the child compiler and library were confirmed to match. An isolated
source-only server build took 8.7 seconds, followed by warm reuse in 2.9 seconds.
These controls did not reproduce the timeout, whose cause remains unconfirmed.
No product code or test timeout was changed. The final full run without parallel
heavy tests passed all 183 tests. The earlier failed log and diagnostic evidence
are retained locally; the timing observation is not presented as a confirmed
namespace regression or a proven contention issue.

Future capability names are allocations only. Byte-buffer design and
implementation remain separate work after AP53.
