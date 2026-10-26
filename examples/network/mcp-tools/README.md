# Independent MCP tools

Run `fpas run examples/network/mcp-tools/mcp-tools.fpasprj`. The program prints
its loopback URL and a confidential Authorization header for your MCP client.
Keep that terminal private and stop with Ctrl+C. Access expires with the process.

`read_value` returns the current value, revision and effect count. `set_value`
requires the expected revision and changes state only when started with
`-- --allow-write`. Stale revisions and denied writes return tool errors without
changing state. The two handlers capture one FPAS state owner and are retained
by registration. HTTP threads never execute application code.

The [shared tools](../mcp-tool-store/src/toolstore.fpas) also serve the
[ACP example](../acp-mcp-tools/README.md). See [Std.Mcp](../../../docs/pascal/std/ai/mcp/README.md)
for serving, cancellation, limits and cleanup.
