# `Std.Ai.OpenAi`

Non-streaming chat completions for configurable OpenAI-compatible HTTP endpoints.

## Quick reference

| Kind | Name | Notes |
|------|------|-------|
| type | `ChatMessage` | role and text content; `System`, `User`, and `Assistant` constructors |
| type | `Client` | base URL, model, optional API key, timeout, and response limit |
| type | `ChatOptions` | optional temperature and maximum token count |
| function | `Complete(Client; Messages; Options): Result of string, string` | returns the first text choice |

```pascal
var ClientValue: Client := Client.Create('http://127.0.0.1:8080/v1', 'local-model');
case Complete(ClientValue, [ChatMessage.User('Hello')], ChatOptions.Default()) of
  Ok(Content): WriteLn(Content);
  Error(Message): panic(Message)
end
```

`Complete` posts JSON to `<BaseUrl>/chat/completions` with `stream: false`. When `ApiKey` is `Some(nonEmpty)`, it sends an `Authorization: Bearer` header. Successful responses must contain text at `choices[0].message.content`. Non-2xx responses and malformed response shapes return `Error(message)`.

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
