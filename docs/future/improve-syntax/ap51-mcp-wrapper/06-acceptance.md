# AP51.6: Verification and live Codex acceptance

Package: [AP51](README.md). Status: Complete.

## Automated verification

Verify public workflows through their real registration and serving operations.
Local peers may be MCP clients; the product interface has the server role.
Automated tests need no account, model access or paid request.

| Path | Required observable result |
| --- | --- |
| Independent `Std.Mcp` program | Register, discover and call tools without ACP or an agent process |
| `Acp.RegisterTool` convenience | Shared registration, automatic advertisement and ACP-owned server cleanup |
| Attached caller-owned server | Existing handlers work through ACP and independent serving survives detach/close |
| Several ACP sessions | Distinct attachment rights; closing one leaves others live |
| Application-owned state | Reads see current values; valid updates return accepted outcomes; stale contexts fail before effects |

Cover SDK/FPAS values, numeric bounds, validation, policy, tool/protocol errors,
cancellation, identity, body/catalog/queue limits, retained captures and teardown.
Use independent peers for wire contracts and FPAS tests for callback execution
and application state. Another test layer needs a distinct transport, lifecycle
or tooling risk.

Verify cancellation and shutdown while ordinary admission is full, no automatic
replay after disconnect, no reuse of terminal rights and no retained native
slots or callbacks after repeated close/reopen. Exercise normal and debugger
hosts. Unrelated ACP work progresses while a tool awaits its application owner.

## Live Codex acceptance

Run locally with the configured Codex account and Luna selected. Record Codex
and ACP-adapter versions, the exact model ID, MCP revision/lifecycle and
transport in delivery evidence. Keep credentials and private paths out of
tracked files and reports.

1. Start the ACP example, register tools through `Acp.RegisterTool` and complete
   normal session setup. Verify real Codex discovery.
2. Ask Codex to read a known value and request an allowed conditional update.
   Observe the requests reach the running FPAS state owner and verify the
   accepted effect count and returned result.
3. Repeat through an existing MCP server attached to ACP. Close ACP and prove
   the caller-owned server remains usable through an independent MCP client.
4. Exercise cancellation and application-context replacement. Old work cannot
   affect the replacement context. Use deterministic peers for invalid,
   denied, stale or malformed cases the model will not reliably produce.
5. Close/reopen attachments and owned servers. Verify retired access identities,
   cleanup and independent ACP progress.

Keep real target evidence separate from fixture coverage. Account or network
failure is not a passing compatibility check. AP51 remains incomplete until the
live discovery/read/update workflow succeeds.

## Regression coverage

| Owner | Contract |
| --- | --- |
| `tests/stdlib/mcp/` | Standalone registration, exact integers, callback suspension, current state, denied/stale/invalid updates, ACP-only imports, borrowed lifetimes and failed-registration rollback |
| `crates/fpas-mcp/tests/` | Offline schemas, admission bounds, one-use replies, cancellation, endpoint isolation, authenticated HTTP and joined runtime cleanup |
| `crates/fpas-acp/tests/tools.rs` | HTTP capability checks, descriptor conflicts, concurrent sessions, failed setup, fresh access and joined detach/close |
| `crates/fpas-cli/tests/batch_debug/mcp.rs` | Registration-only GC roots, resumable debugger callbacks and rejection of foreign mutable captures |
| `crates/fpas-language-service/tests/` | API lookup, declarations, alias variants, signatures and parameter documentation |

Live acceptance covers both registration paths and a conditional update guarded
by the expected revision. The effect counter distinguishes accepted changes
from reads and denied work. An independent MCP client reads the same state after
ACP closes, with no ACP connection/session in its request context. Handlers are
registered once. Target versions and transport are defined in AP51.1; commands,
platform limits and execution results belong in the delivery report.

## Documentation and verification owners

| Action | Path | Purpose |
| --- | --- | --- |
| Create | `tests/stdlib/mcp/`, `crates/fpas-mcp/tests/http.rs`, `crates/fpas-acp/tests/tools.rs` | Independent peers and complete route fixtures |
| Create/modify | `examples/network/mcp-tools/`, `examples/network/acp-mcp-tools/`, `docs/pascal/std/ai/mcp/`, `docs/pascal/std/ai/acp/` | Runnable workflows, setup, errors and ownership |
| Modify | `docs/pascal/std/ai/README.md`, `docs/pascal/std/README.md`, `examples/README.md` | Delivered library and example inventory |
| Generate/check | `lib/api/Std/Mcp.fpas`, `lib/api/Std/Mcp.Types.fpas`, `lib/api/Std/Acp.fpas` | Canonical declarations and callback typing |
| Create/modify | `crates/fpas-cli/tests/batch_debug/mcp.rs`, listener/process tests, `crates/fpas-language-service/tests/` | Retention, cleanup and editor behavior |
| Modify | Package and central AP READMEs | Completion after contract and live acceptance pass |

For implementation, run Rust formatting/build/workspace tests and strict Clippy,
the FPAS suite, affected example checks, generated-declaration checks and
documentation links. Review the contract, repair confirmed findings and repeat
affected checks as needed. Report platform and coverage limits.

Plan edits require link/scope checks and `git diff --check`. Product checks
belong to implementation. Commit, push and merge require the user's instruction.
