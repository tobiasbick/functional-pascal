# AI clients

Protocol clients for model servers. Model execution remains outside the FPAS runtime.

| Unit | Description |
|------|-------------|
| [`Std.Ai.OpenAi`](openai.md) | OpenAI-compatible non-streaming chat completions |
| [`Std.Acp`](acp/README.md) | ACP namespace and opaque connection identities |
| [`Std.Acp.Types`](acp/types.md) | Stable v1 DTOs, lossless values and typed payload codecs |
| [`Std.Acp.Host`](acp/host.md) | Filesystem and terminal payload aliases |
| [`Std.Acp.Experimental`](acp/experimental.md) | Pinned v1 experimental DTOs |
| [`Std.Acp.V2`](acp/v2.md) | Separate v2 DTOs, outcomes and typed payload codecs |

## See also

- [Networking](../network/README.md)
- [Standard library index](../README.md)
