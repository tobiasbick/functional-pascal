# AP51.6: Extensions and ACP boundary

Package: [AP51](README.md). Status: Not started; blocked on AP51.1, AP51.3–AP51.5,
AP51.7/AP51.8 for server integration, and Q61/Q63.

## Contract and scope

Deliver the explicitly selected extension surface for both roles and document exclusions.
The recommendation is an opt-in tasks extension with bounded local tracking,
without MCP Apps, UI hosting or arbitrary automatic extension activation.
Clients consume task results; servers publish and update tasks with bounded
retention and explicit ownership, expiry and cancellation. Tasks belong to
`io.modelcontextprotocol/tasks`, not the core protocol: confirm
the exact extension revision, capability advertisement, task result variants,
methods, polling, cancellation and lifetime rules against the pinned SDK.
Do not transplant the old experimental task protocol.

The existing ACP MCP endpoints remain carriers by default. The same method name
or JSON-RPC shape does not make ACP and MCP identities or authorization scopes
interchangeable. Standalone MCP must work independently of ACP.

If Q63 selects a bridge, specify which side is the MCP client/server, whether the
adapter is a raw transport or an explicit proxy, and who executes tool calls.
Prove that the ACP carrier supports modern MRTR and subscription lifetimes;
unsupported direction or streaming is a feasibility issue to resolve before
implementation. Keep outer ACP IDs, endpoint IDs, inner MCP wire IDs and FPAS
continuation rights separate. Removal of an ACP endpoint or successful parent
session closure must retire every associated bridge request and subscription.
Routing alone grants no tool, filesystem, process or credential authority.

## Affected files and owners

| Action | Path | Purpose |
| --- | --- | --- |
| Create if selected | `crates/fpas-mcp/src/extensions/`, `crates/fpas-mcp/tests/extensions.rs` | Selected extension handling and disabled-feature tests |
| Modify | `crates/fpas-vm/src/vm/hosted/mcp/`, `crates/fpas-sema/src/std_registry/loaded/mcp/` | Explicit opt-ins and task ownership where selected |
| Create if selected | `crates/fpas-vm/src/vm/hosted/mcp/acp.rs`, `crates/fpas-vm/src/vm/hosted/mcp/acp/tests.rs` | A bounded adapter owned by the integration layer, not a crate dependency cycle |
| Inspect/modify only if required | `crates/fpas-acp/src/connection/mcp.rs` | Minimal carrier contract changes backed by bridge tests |
| Create | `docs/pascal/std/ai/mcp/extensions.md` | Actual extension support and the ACP interoperability boundary |
| Modify if bridge selected | `docs/pascal/std/ai/acp/experimental.md` | Document explicit bridge effects and lifecycle |

## Acceptance and verification

Disabled extensions advertise nothing and cannot activate retained state.
Selected task polling/cancellation returns resources on every terminal path.
If bridged, test duplicate inner IDs on separate ACP endpoints, late replies,
parent/child-session retirement, reopened IDs and unrelated ACP progress. Reuse
AP50 carrier/lifecycle regressions. If the bridge is excluded, document that
boundary and verify the standalone wrapper introduces no implicit ACP execution.
Completion requires the selected scope to be delivered, not every optional
alternative in this proposal.
