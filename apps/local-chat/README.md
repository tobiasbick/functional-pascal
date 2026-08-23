# Local Chat

Local Chat is a small `Std.Tui` application written in Functional Pascal. It
connects directly to the local OpenAI-compatible llama.cpp server and uses the
same dark cyan-and-amber visual language as the Notes application.

## Run

Start llama.cpp on `127.0.0.1:11434`, then run:

```sh
fpas run apps/local-chat/local-chat.fpasprj
```

The first version has a deliberately fixed configuration:

- base URL: `http://127.0.0.1:11434/v1`
- model: `unsloth/Qwen3.6-35B-A3B-GGUF:UD-Q4_K_XL`
- one independent prompt per request
- no persisted conversations or context management
- non-streaming responses

Type a one-line prompt and press `Enter` or activate **Send**. `Tab` moves
between the transcript, input, and button. `Alt+X` exits the application. The
interface requires a terminal of at least 50 by 14 cells.

Network requests are synchronous in this initial slice. A slow model therefore
pauses input while a completion is being generated.

## Project layout

```text
apps/local-chat/
├── local-chat-core.fpasprj
├── local-chat.fpasprj
└── src/
    ├── local_chat.fpas
    └── LocalChat/
        ├── Config.fpas
        ├── Model.fpas
        ├── Service.fpas
        ├── Theme.fpas
        ├── Update.fpas
        └── View.fpas
```

The headless workflow regression lives under
[`tests/apps/local_chat/`](../../tests/apps/local_chat/).
