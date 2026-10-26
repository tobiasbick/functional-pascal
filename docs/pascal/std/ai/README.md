# AI clients

Protocol clients for model servers. Model execution remains outside the FPAS runtime.

| Unit | Description |
|------|-------------|
| [`Std.Ai.OpenAi`](openai.md) | OpenAI-compatible non-streaming chat completions |
| [`Std.Acp`](acp/README.md) | Bounded native ACP connections, events and deferred replies |
| [`Std.Acp.Types`](acp/types.md) | Stable v1 DTOs, lossless values and typed payload codecs |
| [`Std.Acp.Host`](acp/host.md) | Explicit file, terminal, interaction and login adapters |
| [`Std.Acp.Experimental`](acp/experimental.md) | Independent feature opt-ins and scoped MCP carriers |
| [`Std.Acp.V2`](acp/v2.md) | Separate v2 negotiation, asynchronous prompts and protocol values |

## See also

- [Networking](../network/README.md)
- [Standard library index](../README.md)
