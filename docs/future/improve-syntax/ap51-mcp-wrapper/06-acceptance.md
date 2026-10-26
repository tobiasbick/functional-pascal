# AP51.6: Verification and Codex acceptance at home

Package: [AP51](README.md). Status: Not started; depends on AP51.1–AP51.5.
Live Codex acceptance is explicitly deferred to home.

## Local automated verification

Test the SDK server binding and ACP route with deterministic local peers.
Fixtures may be MCP clients; this is test infrastructure, not a product role.
Cover values, tool listing/calls, validation, policy, errors, cancellation,
identity, memory bounds and lifetime cleanup.

Mandatory cloud tests require no account, paid model request or live Codex run.
A cloud access limitation is not server failure evidence, and a skipped live
run is not a pass. Reuse meaningful process/listener and GC baseline tests; add
distinct MCP/ACP retention scenarios. Research and necessary benchmarks stay
under `.temp-data/`; cloud timing is not an acceptance criterion.

## Live acceptance at home

1. Record tested Codex/ACP-adapter versions, MCP revision and selected route in
   delivery evidence and current contract documentation.
2. Start FPAS, establish the real ACP session and advertise its MCP server.
3. Observe Codex discover/call a deterministic FPAS tool with known arguments.
   Verify exactly one execution and the correct result returned to Codex.
4. Exercise denied/invalid calls, tool errors and cancellation with observable
   effect counts. Use fixtures for malformed cases Codex will not reliably
   produce, and distinguish those coverage sources.
5. Close/reopen the relevant session/endpoint/application. Old responders stay
   invalid, native resources return to baseline, and unrelated ACP work progresses.

The package remains incomplete until the home compatibility check and live
workflow pass. Do not require Codex to generate every pathological message;
combine real target evidence with deterministic boundary tests. SDK probes and
cloud-only results cannot complete this requirement.

## Documentation and verification owners

| Action | Path | Purpose |
| --- | --- | --- |
| Create | `tests/stdlib/mcp/integration_test.fpas`, `crates/fpas-mcp/tests/interoperability.rs` | Independent local peers and complete route fixtures |
| Create/modify | `examples/network/acp-mcp-tools/`, `docs/pascal/std/ai/mcp/` | Actual setup, route, APIs, errors and limits |
| Modify | `docs/pascal/std/ai/README.md`, `docs/pascal/std/README.md`, `examples/README.md` | Inventory after delivery |
| Generate/check | `lib/api/Std/Mcp.Types.fpas`, `lib/api/Std/Mcp.Server.fpas` | Server declarations and editor coverage |
| Inspect/modify | `crates/fpas-vm/tests/gc_memory.rs`, process/listener tests, `crates/fpas-language-service/tests/` | Resource and tooling boundaries |
| Modify | Package and central AP READMEs | Completion only after all checks, including home acceptance |

For implementation, run Rust formatting/build/workspace tests and strict Clippy,
the FPAS suite, affected examples, generated-API checks and documentation links.
Review the final contract, repair confirmed findings and repeat affected checks
as justified. Report platform/coverage limits.

This plan-only revision needs link/scope checks and `git diff --check`. It does
not authorize product implementation, live experiments, commit or push.
