# AP51: MCP tool registration and ACP integration

Status: Complete. Effort: Medium. Completion is tracked in the
[central plan](../README.md#package-status). This package specifies planned FPAS
library behavior; its checklist records delivery.

## Goal

FPAS applications expose functions as tools through an independent MCP server.
`Std.Mcp` owns tool registration and serving. `Std.Acp` connects to external
agents and offers convenient registration through the same MCP implementation.
An application can use MCP alone, use ACP's convenience interface, or attach an
existing MCP server to an ACP connection.

## Public interface

Keep `Std.Acp` for connections, sessions, prompts, updates and cancellation.
Add `Std.Mcp` for server creation, registration, cooperative serving and
shutdown, with supporting values in `Std.Mcp.Types`.

These call shapes describe the proposed interface. AP51.1 settles complete
types, serving operations and error results before implementation.

```text
Mcp.RegisterTool(Server, Definition, Handler)
Acp.RegisterTool(Connection, Definition, Handler)
Acp.AttachMcpServer(Connection, Server)
```

| Usage | Registration and ownership |
| --- | --- |
| MCP only | An application creates a server, registers tools with `Mcp.RegisterTool` and serves it without ACP or an agent process. |
| ACP convenience | `Acp.RegisterTool` uses a connection-owned MCP server and delegates to the shared registration implementation. ACP advertises it during session setup. |
| Existing server with ACP | `Acp.AttachMcpServer` borrows a caller-owned server and advertises its existing catalog. Definitions and handlers are registered once. |

Both registration paths use the same tool definitions, schemas, validation,
handlers, policy, results and resource limits. Ordinary callers register
functions and use a cooperative serving operation. The library owns the MCP
receive/dispatch/respond loop.

Register tools before publishing the catalog. Keep the catalog fixed while
published; dynamic registration and tool-list change notifications are outside
this package. ACP registration and attachment precede the relevant session
setup request. Each setup receives a distinct attachment identity, including
when several sessions share one catalog. Sharing application state is explicit.

Closing an ACP session retires its attachment and outstanding reply rights.
Closing its connection also closes the ACP-owned convenience server.
Caller-owned servers survive ACP detach or close. Explicitly closing an MCP
server invalidates every attachment to it. Borrowed ownership never becomes
implicit ownership through registration.

## Implementation and transport

Use the official [Rust MCP SDK](https://github.com/modelcontextprotocol/rust-sdk)
with an exact published `rmcp` pin. The SDK owns models, wire serialization,
protocol validation and transport behavior. Compile application-supplied JSON
schemas with the offline `jsonschema` validator before publication. FPAS owns
callable registration, callback execution, policy, admission and resource lifetime.

`Std.Mcp` requires no ACP connection or ACP protocol types. ACP convenience
operations delegate to the MCP implementation. Both registration paths use one
catalog, one schema-validation implementation and one handler execution path.

Local Streamable HTTP is the planned first transport. The listener and tool
handlers belong to the hosting application, so tools can access its live state.
AP51.1 verifies the route, MCP revision and lifecycle against Codex and its ACP
adapter. MCP-over-ACP requires actual target support. Additional transports
require a concrete consumer or evidence that the planned route is unsuitable.

Bind the listener to loopback and define its access policy. Give ACP attachments
distinct access identities and enforce body/message limits before dispatch.
Standalone access has an explicit policy independent of ACP. Keep credentials
out of logs and example output.

## Callable and state contract

A registration contains a name, description, explicit JSON schemas and an FPAS
function. The library validates arguments and invokes the function with request
context. Start with a uniform handler type and structured JSON arguments and
results. Automatic schema derivation from arbitrary FPAS signatures is separate
work.

FPAS tasks execute handlers; the SDK executor handles protocol work. Specify the
owning serving task, cooperative waits and reentry rules. Retain callbacks and
their captures through the VM's GC. Registration grants no concurrent mutable
access to another task's captures.

Applications can forward requests through a bounded channel to their state
owner and await its result. Request context identifies the server, attachment
and call and exposes cancellation. Applications bind any additional context,
check current state and decide which effects are allowed. Closing or replacing
a context cannot redirect its outstanding calls to a new context.

## Scope

Deliver tool discovery/invocation, structured results, tool/protocol errors,
explicit execution policy, cancellation and shutdown. Verify standalone MCP,
ACP convenience registration and attachment of an existing MCP server with
neutral examples and deterministic fixtures.

The product has the MCP server role. Outbound MCP clients, external-server
proxying, resources, prompts, sampling, roots, tasks, MCP Apps, MRTR,
subscriptions, managed OAuth and generic remote hosting are outside this
delivery. Test peers may be MCP clients. Application-specific rules and data
models belong to their applications.

## Decisions

| ID | Topic | Decision or required check | Owner |
| --- | --- | --- | --- |
| Q58 | Role | Reusable FPAS MCP server usable independently of ACP | AP51.1 through AP51.6 |
| Q59 | Public interface | `Std.Mcp`, `Std.Mcp.Types`, both `RegisterTool` entry points and existing-server attachment share one implementation | AP51.1, AP51.2, AP51.5 |
| Q60 | Transport and protocol | Plan local Streamable HTTP in the application process; verify target support, revision and lifecycle | AP51.1, AP51.4, AP51.6 |
| Q61 | Callable execution | Explicit definitions, uniform FPAS handler type and cooperative serving; validation and policy precede effects | AP51.2, AP51.3, AP51.5 |
| Q62 | Authority and lifetime | Bounded work, scoped access, terminal reply rights, VM-owned callbacks and distinct borrowed/ACP-owned server lifetimes | AP51.1, AP51.3, AP51.4 |
| Q63 | ACP integration | Automatic descriptor advertisement during setup, fresh attachment per session and reuse without duplicate registration | AP51.1, AP51.4, AP51.6 |
| Q64 | Implementation | Exact official `rmcp` server SDK pin and one shared MCP implementation | AP51.1 through AP51.6 |

AP51.1 resolves exact signatures, callback servicing, limits and target protocol
details. No FPAS syntax change is planned. Any required syntax or language
semantics change needs explicit user agreement.

## Work packages and dependencies

- [x] [AP51.1: Public contract and target compatibility](01-contract-and-sdk.md)
- [x] [AP51.2: Shared types and public API registration](02-types-and-codecs.md)
- [x] [AP51.3: Owned runtime and FPAS handler dispatch](03-server-runtime.md)
- [x] [AP51.4: Standalone transport and ACP attachments](04-acp-transport.md)
- [x] [AP51.5: Tool registration and application examples](05-tool-execution.md)
- [x] [AP51.6: Verification and live Codex acceptance](06-acceptance.md)

This is the only work-package checklist. AP51.1 defines the executable contract;
AP51.2 and AP51.3 implement types and runtime; AP51.4 supplies transport and
attachments; AP51.5 completes callable registration and examples; AP51.6
verifies all public usage paths.

## Acceptance

A standalone FPAS program serves registered tools without ACP. An ACP program
uses `Acp.RegisterTool` and normal session setup to expose tools to Codex.
Another program attaches an existing MCP server and reuses its handlers.
Closing ACP preserves borrowed servers and releases its owned server resources.

Neutral fixtures verify current application state, accepted effects,
denied/invalid calls, tool errors, stale contexts, cancellation, full queues and
cleanup. Terminal reply rights remain invalid after reopen. Uncertain effects
are never automatically replayed; the transport does not promise exactly-once
application effects.

Live Codex acceptance runs locally with Luna selected. Automated tests need no
account or model access. Record fixture and live evidence separately; target
compatibility remains open until the real discovery/call workflow passes.
Follow the [development process](../development-process.md) and
[shared constraints](../shared-constraints.md).
