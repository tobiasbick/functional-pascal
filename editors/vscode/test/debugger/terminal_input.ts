/** Unit coverage for debugger terminal input decoding. */

import assert from "node:assert/strict";

import { TerminalInputDecoder } from "../../src/debugger/terminal/input";

/** Verify the VT sequences emitted by VS Code's pseudoterminal. */
export function verifyTerminalInputDecoder(): void {
  const decoder = new TerminalInputDecoder();
  assert.deepEqual(decoder.feed("a😀\r\u001b[A\u001b[1;5D\u001b[Z"), [
    { kind: "key", key: "Character", text: "a" },
    { kind: "key", key: "Character", text: "😀" },
    { kind: "key", key: "Enter" },
    { kind: "key", key: "Up", shift: false, alt: false, ctrl: false, meta: false },
    { kind: "key", key: "Left", shift: false, alt: false, ctrl: true, meta: false },
    { kind: "key", key: "Tab", shift: true, alt: false, ctrl: false, meta: false }
  ]);
  assert.deepEqual(decoder.feed("\u001b[<0;12;7M"), [
    {
      kind: "mouse",
      action: "Down",
      button: "Left",
      x: 12,
      y: 7,
      shift: false,
      alt: false,
      ctrl: false,
      meta: false
    }
  ]);
  assert.deepEqual(decoder.feed("\u001b[200~multi"), []);
  assert.deepEqual(decoder.feed(" line\u001b[201~"), [
    { kind: "paste", text: "multi line" }
  ]);
}
