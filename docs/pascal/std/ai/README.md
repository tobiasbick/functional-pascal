# AI protocols

Protocol clients for model servers. Model execution remains outside the FPAS runtime.

| Unit | Description |
|------|-------------|
| [`Std.Ai.OpenAi`](openai.md) | OpenAI-compatible non-streaming chat completions |
| [`Std.Ai.Acp`](acp/README.md) | Bounded native ACP connections, events and deferred replies |
| [`Std.Ai.Acp.Types`](acp/types.md) | Stable v1 DTOs, lossless values and typed payload codecs |
| [`Std.Ai.Acp.Host`](acp/host.md) | Explicit file, terminal, interaction and login adapters |
| [`Std.Ai.Acp.Experimental`](acp/experimental.md) | Independent feature opt-ins and scoped MCP carriers |
| [`Std.Ai.Acp.V2`](acp/v2.md) | Separate v2 negotiation, asynchronous prompts and protocol values |
| [`Std.Ai.Mcp`](mcp/README.md) | Registered FPAS tools over authenticated loopback HTTP, independently or through ACP |
| [`Std.Ai.Mcp.Types`](mcp/types.md) | Shared tool definitions, request context, results and errors |

## See also

- [Networking](../network/README.md)
- [Standard library index](../README.md)
