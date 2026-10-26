# AP51.4: ACP-advertised MCP transport

Package: [AP51](README.md). Status: Not started; depends on AP51.1–AP51.3.

## Contract

Make the SDK-backed FPAS server reachable through the Codex route verified at
home in AP51.1. Advertise its descriptor in ACP session setup and enforce peer
capability requirements. ACP integration is required; MCP-over-ACP is selected
only if the actual target supports it.

| Selected route | Conditional implementation work |
| --- | --- |
| MCP-over-ACP | Adapt existing endpoint/carrier events to the SDK server; separate ACP/MCP IDs and return through the matching responder |
| Local HTTP | Own a loopback listener and official SDK service; advertise its URI and apply the selected revision, limits and access policy |
| Stdio process | Own protocol stdin/stdout in a launched FPAS server; provide its descriptor and define the tool-state process boundary |

Choose the smallest suitable route, not all three. Use SDK protocol/transport
facilities with FPAS lifetime adapters. Do not add an outbound MCP client or
external-server proxy. Configure discovery, initialization and HTTP behavior
from the verified target revision.

The selected server must reach the intended application/tool state. A stdio
child cannot silently share host memory; reconsider the route before expanding
scope into additional IPC. Keep protocol streams separate from console,
diagnostic and debugger output where applicable.

Apply explicit policy before effects. Bind any route credentials/local access
to the intended authority. The local Codex workflow does not require managed
OAuth, an authorization server or generic remote hosting.

## Implementation owners

| Action | Path | Purpose |
| --- | --- | --- |
| Create | `crates/fpas-vm/src/vm/hosted/mcp/acp.rs` and tests | ACP setup/routing and lifetime integration |
| Create as selected | `crates/fpas-mcp/src/transport/` | Official SDK carrier/HTTP/stdio adapter |
| Inspect/modify as needed | `crates/fpas-acp/src/connection/mcp.rs` | Minimal carrier repairs backed by evidence |
| Inspect/reuse as selected | `crates/fpas-vm/src/vm/hosted/net/listeners.rs`, `crates/fpas-vm/src/vm/hosted/server/`, CLI/console stream owners | Stream/listener ownership and shutdown |
| Create/modify | `docs/pascal/std/ai/mcp/transports.md`, `docs/pascal/std/ai/acp/experimental.md` | Actual route and effects |

## Acceptance

Fixtures verify descriptor setup, IDs, unsupported capabilities, isolation,
disconnect and cleanup. Removal/reopen invalidates associated work permanently.
For a separate HTTP/stdio route, explicitly own its relation to the ACP
session/application lifetime. Live Codex verification occurs at home in AP51.6;
fixture routing does not itself establish target compatibility.
