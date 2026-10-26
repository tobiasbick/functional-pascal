# Application tools through ACP

Run an installed, authenticated ACP agent with an explicit working directory:

```text
fpas run examples/network/acp-mcp-tools/acp-mcp-tools.fpasprj -- /path/to/agent /absolute/workspace "Read the current value using read_value" --stdio
```

Use Windows executable and directory paths on Windows. Arguments after the
prompt go to the agent. The example neither installs an agent nor selects a
provider or model. Configure the agent separately; local Codex acceptance uses
`gpt-6-luna`. File and terminal capabilities remain disabled.

By default `Acp.RegisterTool` owns the shared application's MCP server. `--attach`
instead registers the same handlers through `Mcp.RegisterTool`, borrows that
server with `Acp.AttachMcpServer`, and confirms it remains publishable after ACP
closes. The MCP library handles advertisement, HTTP access and dispatch.

`--allow-write` enables the application's conditional update policy. Ask the
agent to read the revision, call `set_value` with a matching revision, then read
the accepted value and effect count. ACP permission requests display the offered
options and wait for your selection; an empty response cancels. Application
policy still checks every update. Protocol permission alone never changes state.

The state owner serves callbacks while a separate FPAS task runs the ordinary
ACP event loop. Closing ACP joins its HTTP publications. Cancellation stops
serving; releasing the caller-owned server releases its registered captures.
