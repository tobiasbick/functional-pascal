# 02: shared read-only inspection

Depends on [01](01-contract.md). Progress lives in the [overview](README.md).

## Goal and deliverable

Extract the common read-only frame, binding, location and value inspection used
by the existing debugger, making it usable at a normal worker's fault boundary.
Keep debugger stop generations, mutation targets and variable handles in the
debugger. Avoid creating a second type renderer or lexical-visibility resolver.

## Implementation boundaries

Inspect [frame capture](../../../crates/fpas-vm/src/vm/debug/inspection/capture.rs),
[typed bindings](../../../crates/fpas-vm/src/vm/debug/inspection/typed_bindings.rs),
[value rendering](../../../crates/fpas-vm/src/vm/debug/inspection/render.rs),
[worker state](../../../crates/fpas-vm/src/vm/worker.rs) and
[register initialization](../../../crates/fpas-vm/src/vm/register_stack.rs).

Introduce focused modules under `crates/fpas-vm/src/vm/inspection/` for the
actual common operations, with debugger adapters staying in their existing
directory. Confirm the exact move/create paths before implementing. Avoid a
new crate or public interface that exists only for tests. VM-facing inspection
must read worker-local globals, initialized registers, references, callback
continuations and capture cells correctly under the contract's budgets.

The failure report must own stable data. Sharing a live mutable cell or a
debugger handle into a later serialization phase is not a snapshot. Capture
uses nonblocking access to shared state and never invokes FPAS code.

## Acceptance criteria

- Parameters, shadowed locals, captures, `var` parameters and globals resolve
  to the actual visible values; unavailable and uninitialized values differ.
- Caller positions and active instruction positions are correct in normal
  execution, including failure at a hosted callback boundary.
- Aggregates, cycles, missing metadata and each applicable value bound follow
  the version-1 value contract without unbounded intermediate allocation.
- Ordinary debugger stack/scopes/variables and DAP/JSONL behavior retain their
  current semantics. Mutable debugger features continue through their adapters.
- No successful-instruction tracing or failure-capture activation is introduced
  as a side effect of the refactor.

## Verification to record on delivery

Use the existing VM debugger inspection and aggregate tests as their primary
owners, adding cases for normal workers only where debugger tests cannot
exercise the different register/global/callback lifecycle. Run the affected
VM/debugger tests, formatting, build and required workspace checks. Record
actual commands, results and any limitations here when implementing this step.

## Implementation

The shared implementation lives under
[`vm/inspection`](../../../crates/fpas-vm/src/vm/inspection):

- `frames.rs` visits frame windows without first cloning an entire stack. A
  stopped debugger uses the next instruction; a normal fault uses the last
  dispatched instruction. Saved callers use the call site. Failed synchronous
  callback workers precede their calling continuation; task-owned inline
  callbacks already retain their caller in the ordinary stack. Busy callback
  storage returns an unknown frame count and an explicit reason.
- `location.rs`, `bindings.rs` and `globals.rs` share lexical visibility, portable
  positions and initialization-aware reads. Global inspection selects the actual
  local/shared storage, and reference/cell inspection never waits on a lock.
- `values/` owns the common scalar formatting, typed structural selectors,
  bounded lazy children and detached `ValueSnapshot` trees. The payload resolver
  and its existing tests moved from the debugger into this directory. Debugger
  mutation origins implement the shared origin adapter; handles, generations,
  assignment and evaluation state remain in the debugger.
- Detached snapshots contain owned scalar text and child trees, without runtime
  `Value` storage, mutable cells or debugger origins. A caller-supplied value/byte
  budget is shared across a capture. Unicode payloads and summaries have separate
  character bounds so a dump can bound all captured text while the debugger keeps
  its existing quoting behavior. Child bounds apply before materialization;
  dictionary entries retain whole key/value pairs.

Normal execution still uses its ordinary scheduler. This step does not register
failure candidates, retain frames across scope unwind, collect peer tasks, or
enable a CLI option. Those integrations remain steps 03–05. The cached callback's
outcome marker is updated only at callback completion; no instruction tracing,
capture allocation or collection polling was added to successful execution.

The permanent language, CLI, authoring-reference and debugger documentation needs
no update for this internal refactor. Existing debugger entry points and protocols
remain unchanged.

## Verification evidence

Nine new owner-level tests cover normal-worker globals, fault/caller positions,
shadowing, `var` referents, escaped capture cells, both hosted callback paths,
contention, detached values, cycles, missing metadata and allocation bounds.
The existing debugger inspection, mutation and protocol suites remain the owners
of their user-facing behavior. The two payload resolver tests moved with their
implementation rather than being duplicated.

The early-allocation regression was checked by temporarily restoring the old
array enumeration order: `aggregate_limits_apply_before_child_origins_are_materialized`
failed with 10,000 materialized origins instead of 2. Restoring the bound made it
pass. All 11 shared-inspection tests passed before the workspace checks.

Checks run:

- `cargo fmt --all` and `cargo fmt --all -- --check`: passed. The final two changed
  value files were also formatted directly after the last boundary-test addition.
- `cargo build`: passed again after the final inspection changes.
- `cargo test -p fpas-vm vm::inspection --lib --quiet`: all 11 tests passed.
- `cargo test --workspace --quiet`: passed through the CLI/FPAS documentation,
  debugger DAP/JSONL and preceding crate suites, then stopped at the existing
  handbook keyword test. Its fence reader counted the `text` info string as a
  keyword after the earlier documentation migration. The existing owner test now
  skips the fence's info line; no keyword inventory or language rule changed.
- The corrected lexer and remaining crates were checked with:

  ```sh
  cargo test -p fpas-lexer -p fpas-linker -p fpas-lsp -p fpas-numeric \
    -p fpas-parser -p fpas-program -p fpas-project -p fpas-sema \
    -p fpas-std -p fpas-unit -p fpas-vm --quiet
  ```

  All passed except two previously reproduced network VM tests:
  `tcp_backpressure_can_time_out_and_cancel_without_closing` received a successful
  write instead of cancellation, and
  `write_deadline_is_not_reset_by_retries_and_preserves_socket_timeouts` observed
  a rounded 32 ms socket timeout instead of 30 ms. Neither owner changed here.
- The final VM check excluded only those two existing failures:

  ```sh
  cargo test -p fpas-vm --quiet -- \
    --skip tcp_backpressure_can_time_out_and_cancel_without_closing \
    --skip write_deadline_is_not_reset_by_retries_and_preserves_socket_timeouts
  ```

  Result: 622 unit tests and 9 integration tests passed; 2 unit tests filtered.
- `cargo test --workspace --doc --quiet`: all 3 Rust documentation examples passed.
- `cargo clippy -p fpas-vm --lib --no-deps --quiet`: passed with only the two
  existing `chunks_exact_to_as_chunks` warnings in `execute/aggregates/records.rs`.
- Relative Markdown links and `git diff --check`: passed.

No standalone full FPAS suite rerun was needed: no `.fpas` source changed, and
Cargo ran its existing FPAS themes and documentation-example checks. The new
normal-worker fixtures compile and execute FPAS inside their Rust owner tests.
No performance or complete failure-dump delivery is claimed by this step.
