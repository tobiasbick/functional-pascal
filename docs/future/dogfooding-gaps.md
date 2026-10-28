# Dogfooding gaps

Intake list for gaps found while agents write their helper scripts in FPAS instead of Python,
JavaScript, or shell logic.
Compiler panics also follow the entry rule in
[compiler-panic-followups.md](compiler-panic-followups.md).

## Entry format

Add one entry per gap under [Open entries](#open-entries):

```markdown
### <Short title>

- **Date:** YYYY-MM-DD
- **Kind:** blocker (fell back to another language) or awkward (done in FPAS)
- **Area:** language, `Std.*` API, bug, diagnostics, or performance
- **Task:** what the helper script had to do
- **Missing or wrong:** what FPAS could not do or did badly
- **Minimal example:** the smallest FPAS source that shows the gap
- **Fallback:** the language or tool used instead, if any
```

Remove an entry once the gap is fixed and covered by tests and current documentation.

## Open entries

### Batch debugger scripts cannot feed unified Console events

- **Date:** 2026-10-28
- **Kind:** awkward (done in FPAS)
- **Area:** debugger scripts, `Std.Console`, `Std.Tui`
- **Task:** Run the actual fpastris entry point and quit through scripted keyboard
  input in a bounded `fpas debug` invocation.
- **Missing or wrong:** `readkey_chars` fills the legacy character queue;
  `Console.ReadEventTimeout`, used by the TUI input pump, reads the separate
  unified event queue. The script format exposes no unified key or resize event.
  Queuing `q` therefore leaves the application waiting until the debug timeout.
- **Minimal example:** Run `fpas debug apps/fpastris/fpastris.fpasprj --script
  quit.script.toml --timeout 15` with `[[event]]`, `type = "readkey_chars"` and
  `chars = "q"` in the script.
- **Fallback:** FPAS headless tests use `TuiApplication.OpenForTestWithPalette`,
  `InjectKeyForTest`, `InjectResizeForTest` and `RunIterations` to verify input,
  rendering and lifecycle. This does not exercise the native terminal pump.

### Model-token counts need an external tokenizer

- **Date:** 2026-10-25
- **Kind:** awkward (helper written in FPAS)
- **Area:** `Std.*` API
- **Task:** Verify the agent reference's total token budget and entrypoint size.
- **Missing or wrong:** FPAS has no model tokenizer for exact BPE counts; byte,
  scalar and word counts cannot establish the requested token limit.
- **Minimal example:** `Text.Length()` counts Unicode scalars, while the required
  count is `o200k_base` tokens. The FPAS helper passes the text to
  `Proc.RunCaptureWithOptions` with `Input := Text`.
- **Fallback:** The installed Python `tiktoken` library performs encoding through
  that child process; file discovery, reporting and budget checks stay in FPAS.
