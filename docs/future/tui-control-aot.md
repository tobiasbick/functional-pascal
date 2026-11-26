# TUI control publication gap

The Native AOT TUI gallery runs in a terminal. When a controller connects,
the control protocol fails while serializing its first frame. The protocol
uses reflection-based JSON serialization for frames and replies, and the
Native AOT build disables it.

Use the managed gallery build for the existing headless tests. The tests
verify frames and input through the control pipe. Start the Native AOT
executable in a private terminal to verify the interactive build.

To support the control protocol in Native AOT, give frames and replies explicit
serialization metadata. Verify the same frame, input and reply contracts in a
published Native AOT build.

- [TUI gallery launch instructions](../../examples/ui/tui/gallery/README.md#run-the-gallery)
- [TUI control transport](../../src/Fpas.Processes/Terminal/TuiControlConnection.cs)
