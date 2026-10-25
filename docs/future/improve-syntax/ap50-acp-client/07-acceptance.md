# AP50.7: Distribution, examples and whole-client acceptance

Package: [AP50](README.md). Status: planned; implementation not started.

## Prerequisites

AP50.1 through AP50.6 are delivered on the selected branch with their own tests
and current documentation. This package proves composition and distribution;
it is not where the earlier packages first acquire contract coverage.

## Deliverable and scope

Ship a discoverable client library that an ordinary FPAS program can use with
a separately implemented agent. Complete the module handbooks, source/API
distribution, CLI doc lookup and editor import/navigation behavior. Confirm
all required entries in [protocol-coverage.md](protocol-coverage.md) through
their owning tests and the whole-client scenarios below.

Provide `examples/network/acp-client/` with a small console event-loop program.
Agent command/args/cwd are explicit application inputs. The loop prints content
and tool progress, lets the user answer requests, and demonstrates cancellation
and cleanup. It defaults to denying file/terminal effects unless the user has
selected the corresponding example option. Do not hardcode a provider, install
an agent, save credentials or call external services during automated checks.

Add a second example selecting v2 and explicit experimental options. Explain
the difference between prompt acceptance and completion. Keep examples small
by sharing ordinary FPAS application logic, rather than adding an ACP-specific
CLI framework or production proxy.

## Independent peers and fixture ownership

Use hand-authored JSON message fixtures derived from the pinned protocol, not
the binding's own encoder as the expected value. A scripted FPAS subprocess
peer sends these messages over actual stdio and verifies client replies.
It is test support, not a production Agent interface. Its source is outside the
test bundle's `*_test.fpas` discovery pattern and receives explicit scenarios.

Use a separately implemented protocol peer for whole-client interoperability.
The scripted peer's encoder/parser must be independent of `fpas-acp`; fixed
wire fixtures suffice for this controlled verification. An additional smoke
run with an installed third-party ACP agent can provide compatibility evidence.
It is conditional on an available authorized installation/authentication and
must not become the only acceptance gate or require external secrets in CI.

Rust tests own adapter-specific framing/codec/queue cases. Public FPAS tests
own application-visible contracts. CLI subprocess tests own the distinct
distributed-runtime, stdio, exit-status and native-executable risks. Reuse
fixtures where they represent the same wire contract, but do not replay all
codec cases at every layer or introduce test-only production exports.

## Affected files and implementation owners

| Action | Path | Purpose |
| --- | --- | --- |
| Create | `examples/network/acp-client/main.fpas`, `acp-client.fpasprj`, `README.md` | Stable client example and running instructions |
| Create | `examples/network/acp-client/v2.fpas`, `acp-v2.fpasprj` | Explicit v2/experimental example |
| Create | `tests/support/acp/scripted_agent.fpas`, `fixtures/` | Independent deterministic process peer and fixed messages |
| Create | `crates/fpas-cli/tests/acp.rs`, `acp/` | Public CLI/process/distribution acceptance |
| Create | `tests/stdlib/acp/integration_test.fpas` | Complete ordinary-program client scenario |
| Modify | `lib/stdlib.fpasprj`, distribution/API staging owners under `crates/fpas-cli/` and `crates/fpas-build/` | Export any source modules, stage all generated declarations and matching artifacts |
| Modify | `docs/pascal/std/ai/README.md`, `docs/pascal/std/README.md`, `examples/network/README.md` | Discovery and example routing |
| Create/finalize | `docs/pascal/std/ai/acp/README.md` and topic pages delivered earlier | Self-contained client handbook and links |
| Modify | `docs/pascal/agents/tools.md` | Small ACP routing entry, not the entire protocol reference |
| Verify/regenerate | `lib/api/Std/Acp.fpas`, `Acp.Types.fpas`, `Acp.Host.fpas`, `Acp.Experimental.fpas`, `Acp.V2.fpas` | Complete declarations under `lib/api/Std/` and handbook comments |

New native modules are not source exports in `lib/stdlib.fpasprj`. Add only
actual source modules to that manifest; stage intrinsic declarations through
the existing generated API path. Do not create two implementations with the
same canonical Std unit name.

## Whole-client acceptance scenarios

1. Connect over real stdio, initialize/authenticate, configure cwd/roots/MCP,
   create two sessions, send multimodal prompts and receive interleaved content,
   tool, plan/config/info/usage, notice and compaction updates. Replies retain
   IDs and raw metadata. Load/replay, resume, list pagination, close, delete
   and logout all obey capability gates and their separate lifecycle rules.
2. During a prompt, handle permission, form and URL requests, provide an unsaved
   file buffer, perform an explicitly allowed disk operation, create a terminal,
   obtain bounded output/status, kill and release it. An unrelated session
   remains responsive while another request waits on the application.
3. Cancel a session with pending permissions, cancel an independent request,
   time out a receive, then process valid later events and final results.
   Refused/stale responses never cause host effects. Closing a connection
   during active child work leaves no child, responder or driver alive.
4. Repeat the applicable v2 scenario with acceptance distinct from idle
   completion, resume replay, v2 permissions/content/tool/terminal updates,
   scoped MCP routing and each enabled experimental feature. Repeat feature
   guards with each option disabled and with missing peer support.
5. Run from a fresh staged standard library and from reused source-adjacent
   artifacts, including a built native executable. Compile a separate consumer
   importing each public module. Doc lookup, hover, completion and definitions
   resolve the same public names/types as compilation.

These scenarios prove composition. Reuse the existing focused boundary cases
for malformed JSON, numeric precision, queue exhaustion and process races;
do not duplicate their exhaustive matrices in the large end-to-end scenario.

## Verification commands and completion

Run focused owning tests first, then the applicable required checks:

```text
cargo run -p fpas-sema --example export_intrinsic_std_api
cargo fmt --all -- --check
cargo build
cargo test --workspace
fpas fmt --check lib apps examples tests
fpas test tests/suite.fpasprj
fpas check examples/network/acp-client/acp-client.fpasprj
fpas check examples/network/acp-client/acp-v2.fpasprj
git diff --check
```

Validate handbook relative links and all executable examples. Reuse the compact
reference's existing example test and the documentation-example workflow.
Verify the generated declarations and current handbooks agree. Run the owning
editor tests when registration/import/navigation behavior changes; syntax
highlighting is unaffected unless implementation uncovers an actual change.

Run process/status/terminal/cleanup acceptance on Windows and Unix. Report
which platforms and independent peers were actually exercised, without treating
an unrun platform as verified. Inspect native teardown after ordinary completion,
runtime failure and debugger disconnect. Use `fpas debug` for FPAS failures
and `fpas run --profile` for FPAS performance investigations before fallbacks.

Review against Q55, this interface, the pinned inventory, native ownership and
test authoring rules. Fix confirmed findings at their owner and rerun relevant
checks. The delivery report records the fixed baseline/target revisions,
commands, results and remaining coverage limits. Keep chronological test logs
and machine metadata out of the plan.

Only after all seven work packages and the package acceptance are delivered
may the package README and central AP row be checked off. Commit, push and
merge remain separate user-authorized actions. Update or remove completed
planning files according to the AP workflow after final delivery.
