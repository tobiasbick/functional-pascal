# `Std.Ai.OpenAi`

Non-streaming chat completions for configurable OpenAI-compatible HTTP endpoints.

## Quick reference

| Kind | Name | Notes |
|------|------|-------|
| type | `ChatMessage` | role and text content |
| function | `ChatMessageCreate(Role; Content)` | constructs a message with an explicit role |
| function | `ChatMessageSystem`, `ChatMessageUser`, `ChatMessageAssistant` | role-specific constructors; each takes `Content` |
| type | `Client` | base URL, model, optional API key, timeout, and response limit |
| function | `ClientCreate(BaseUrl; Model)` | constructs a client with default limits |
| type | `ChatOptions` | optional temperature and maximum token count |
| function | `ChatOptionsDefault()` | supplies default generation options |
| function | `Complete(Client; Messages; Options): Result of (string, string)` | returns the first text choice |

```pascal
uses Std.Console as Console;
uses Std.Ai.OpenAi as AiOpenAi;

const ClientValue: AiOpenAi.Client := AiOpenAi.ClientCreate('http://127.0.0.1:8080/v1', 'local-model');
case AiOpenAi.Complete(ClientValue, [AiOpenAi.ChatMessageUser('Hello')], AiOpenAi.ChatOptionsDefault()) of
  when Result.Ok(const Content):
    Console.WriteLn(Content);
  when Result.Error(const Message):
    panic(Message);
end case;
```

`Complete` posts JSON to `<BaseUrl>/chat/completions` with `stream: false`. When `ApiKey` is `Option.Some(nonEmpty)`, it sends an `Authorization: Bearer` header. Successful responses must contain text at `choices[0].message.content`. Non-2xx responses and malformed response shapes return `Result.Error(message)`.

`Complete` remains buffered. Callers that need token delivery can build a streaming request with
[`Std.Http.OpenStream` and its SSE decoder](../network/http.md#streaming-responses).

The runnable chat project keeps user and assistant messages in memory:

```text
fpas run examples/openai-chat/openai-chat.fpasprj -- http://127.0.0.1:8080/v1 local-model
```

Set `OPENAI_API_KEY` in the process environment when an endpoint requires authentication. The example does not print the key.

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Public facade | [`OpenAi.fpas`](../../../../lib/Std/Ai/OpenAi.fpas) |
| HTTP client orchestration | [`Client.fpas`](../../../../lib/Std/Ai/OpenAi/Client.fpas) |
| JSON wire format | [`Json.fpas`](../../../../lib/Std/Ai/OpenAi/Json.fpas) |
| End-to-end fixture | [`network.rs`](../../../../crates/fpas-cli/src/main_tests/network.rs) |

## See also

- [AI client index](README.md)
- [`Std.Http`](../network/http.md)
