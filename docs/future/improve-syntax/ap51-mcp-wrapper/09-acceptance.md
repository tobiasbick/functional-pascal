# AP51.9: Interoperability, documentation and acceptance

Package: [AP51](README.md). Status: Not started; blocked on all selected delivery
work in AP51.1–AP51.8.

## Deliverable

Demonstrate both client and server contracts using independent peers, ship usable
FPAS examples and document the actual capability/transport/authentication
matrix. A successful same-SDK round trip is insufficient interoperability evidence.

## Affected files and owners

| Action | Path | Purpose |
| --- | --- | --- |
| Create | `examples/network/mcp-client/` | Runnable stdio and HTTP projects with explicit interaction handling |
| Create | `examples/network/mcp-server/` | Runnable FPAS tool/resource/prompt server with explicit shutdown and authorization configuration |
| Create | `tests/stdlib/mcp/integration_test.fpas`, `crates/fpas-mcp/tests/interoperability.rs` | Reproducible independent-peer integration |
| Modify | `crates/fpas-vm/tests/gc_memory.rs` and owning process tests | Reuse allocation-baseline and native resource checks for MCP ownership |
| Create/modify | `docs/pascal/std/ai/mcp/README.md` and its topic pages | Delivered APIs, examples, errors, limits and exclusions |
| Modify | `docs/pascal/std/ai/README.md`, `docs/pascal/std/README.md`, `README.md`, `examples/README.md` | Discoverability and supported-library inventory |
| Generate | `lib/api/Std/Mcp*.fpas` for the agreed units | Canonical editor declarations and documentation |
| Inspect/modify | `docs/pascal/agents/`, `.agents/skills/fpas-authoring/`, `crates/fpas-language-service/tests/` | Authoring guidance and native API/editor coverage where affected |
| Modify | `docs/future/improve-syntax/ap51-mcp-wrapper/README.md`, `docs/future/improve-syntax/README.md` | Record delivery completion after verification |

Select and pin an independently implemented peer or conformance harness during
AP51.1. Mandatory tests use local reproducible fixtures, not external accounts,
paid APIs, floating downloads or caller secrets. Exploratory evidence and any
necessary benchmarks stay under `.temp-data/`; cloud timings are not performance
acceptance criteria. Do not create a broad benchmark campaign for this wrapper.

## Acceptance matrix

| Boundary | Required evidence |
| --- | --- |
| Protocol | Agreed revision, optional discovery, per-request metadata, selected capabilities, result/error variants and rejected unknown variants |
| Transports | Independent stdio and HTTP peers, JSON and SSE, routing headers, framing, limits and authentication behavior |
| Server | Independent clients, dispatch/response rights, simultaneous callers, request-state integrity, listener/stdio ownership and authorization before effects |
| Interactions | Explicit MRTR, separate IDs/state per operation, subscriptions, progress and selected extensions |
| Ownership | Repeated connection/admission failures, process reaping, stale handles, cancellation races, quiet/full-queue shutdown and memory baselines |
| Runtime | Normal execution and debugger hosts, cooperative task cancellation, GC roots, no FPAS callbacks on the native executor |
| Tooling | Generated declarations, names/signatures, editor completion/navigation, source and rebuilt/reused artifacts |
| Coexistence | Existing ACP sessions, carriers, host adapters and process tests remain green |

Review the completed diff against the approved contract and repository standards,
confirm findings with reproductions, repair them and rerun the affected checks.
Record any platform coverage limitations in the delivery report.

Run `cargo fmt --all -- --check`, `cargo build --workspace`,
`cargo test --workspace`, and
`cargo clippy --workspace --all-targets --all-features -- -D warnings`.
Run `fpas test tests/suite.fpasprj`, check the affected example projects, validate
generated declarations and documentation links, and run `git diff --check`.
Use the project formatter for changed FPAS sources. Additional full-suite reruns
need new changes, failures or unresolved concerns.

AP51 is complete only when the selected scope is delivered, documented and
verified and all work-package boxes can truthfully be checked. Git integration
and removal of the completed plan follow separate user authorization.
