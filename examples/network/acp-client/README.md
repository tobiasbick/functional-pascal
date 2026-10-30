# ACP console clients

Run a separately installed ACP agent with an explicit executable, working
directory and prompt. Both examples handle `--allow-host` themselves; other remaining
arguments are passed directly to the executable. There is no shell, package installation
or provider-specific setup in the example.

```text
fpas run examples/network/acp-client/acp-client.fpasprj -- /path/to/agent /absolute/workspace "Explain this project" --stdio
fpas run examples/network/acp-client/acp-v2.fpasprj -- /path/to/v2-agent /absolute/workspace "Explain this project" --stdio
```

Use Windows executable and absolute directory paths on Windows. The executable
must speak newline-delimited ACP on stdio. Authenticate the agent separately
before running this example. [Terminal authentication](../../../docs/pascal/std/ai/acp/authentication.md)
explains how an application can implement an interactive login flow.

The shared [event loop](demo.fpas) prints protocol content and tool progress,
asks for a permission option or elicitation response, and closes/releases the
connection. Empty interaction input cancels. It displays URL requests without
opening a browser. After submitting a prompt, enter `cancel` to send session
cancellation, or Enter to start displaying the stream. The driver continues
while the console waits for a decision.

File and terminal capabilities are disabled by default. Add `--allow-host` to
advertise them in the v1 example. Every host operation then also asks for `yes`
before invoking the corresponding typed adapter. The example does not implement
a path sandbox. A host adapter failure ends the program; VM teardown still owns
connection/terminal cleanup. Permission selection alone does not execute a host
operation.

The v2 example explicitly requests v2 and enables experimental token usage.
It waits separately for prompt acceptance and the foreground idle update.
Background tools and messages have their own lifetimes. It uses the v2 message
and permission shapes and offers no v1 file/terminal capability. Peers selecting
a lower version are demonstrated in the public regression test, where the
caller explicitly switches families without resending initialization.

No automated example check contacts an external service. Independent stdio
acceptance uses [scripted_agent.fpas](../../../tests/support/acp/scripted_agent.fpas),
which parses fixed wire messages with `Std.Encoding.Json` independently of the ACP binding.
See the [client handbook](../../../docs/pascal/std/ai/acp/README.md) for bounded
queues, deferred replies, authentication, MCP endpoints and session policy.
