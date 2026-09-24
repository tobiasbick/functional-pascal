import assert from "node:assert/strict";

import { TerminalInputDecoder, MAX_TERMINAL_INPUT_BYTES, type TerminalInputEvent } from "../../src/debugger/terminal/input";
import { TerminalInputQueue } from "../../src/debugger/terminal/inputQueue";
import { TimedTerminalInput } from "../../src/debugger/terminal/timedInput";

/** Checks all string boundaries, unsupported-sequence recovery and bounded delivery. */
export async function verifyTerminalBoundaries(): Promise<void> {
  for (const sequence of ["a😀z", "\u001b[A", "\u001b[1;5D", "\u001bOP", "\u001bOA",
    "\u001b[I", "\u001b[O", "\u001b[<35;12;7M", "\u001b[<0;12;7m", "\u001b😀",
    "\u001b[200~a😀\u001b[A text\u001b[201~"]) {
    const expected = new TerminalInputDecoder().feed(sequence);
    for (let split = 0; split <= sequence.length; split += 1) {
      const decoder = new TerminalInputDecoder();
      assert.deepEqual([...decoder.feed(sequence.slice(0, split)), ...decoder.feed(sequence.slice(split))], expected,
        `${JSON.stringify(sequence)} split ${split}`);
    }
    const decoder = new TerminalInputDecoder();
    assert.deepEqual(sequence.split("").flatMap(character => decoder.feed(character)), expected);
  }
  const decoder = new TerminalInputDecoder();
  assert.deepEqual(decoder.feed("\u001b[99~abc"), new TerminalInputDecoder().feed("abc"));
  assert.deepEqual(decoder.feed("\u001b"), []);
  assert.deepEqual(decoder.flushEscape(), [{ kind: "key", key: "Escape" }]);
  for (const [code, suffix, action, button] of [
    [35, "M", "Move", "None"], [32, "M", "Drag", "Left"], [32, "m", "Up", "Left"],
    [0, "m", "Up", "Left"], [64, "M", "ScrollUp", "None"], [65, "M", "ScrollDown", "None"]
  ] as const) {
    const [event] = decoder.feed(`\u001b[<${code + 4 + 8 + 16};12;7${suffix}`);
    assert.deepEqual(event, { kind: "mouse", action, button, x: 12, y: 7,
      shift: true, alt: true, ctrl: true, meta: false });
  }
  assert.throws(() => decoder.feed("\u001b[" + "1".repeat(129)), /128/u);
  assert.deepEqual(decoder.feed("a"), [{ kind: "key", key: "Character", text: "a" }]);
  decoder.feed("\u001b[200~");
  decoder.feed("a".repeat(MAX_TERMINAL_INPUT_BYTES));
  assert.throws(() => decoder.feed("b"), /paste exceeds/u);
  assert.deepEqual(decoder.feed("b"), [{ kind: "key", key: "Character", text: "b" }]);

  const timedEvents: TerminalInputEvent[] = [];
  const timed = new TimedTerminalInput(events => timedEvents.push(...events), error => { throw error; });
  timed.feed("\u001b");
  await new Promise(resolve => setTimeout(resolve, 80));
  assert.deepEqual(timedEvents, [{ kind: "key", key: "Escape" }]);
  timed.feed("\u001b");
  timed.dispose();
  await new Promise(resolve => setTimeout(resolve, 80));
  assert.equal(timedEvents.length, 1);

  for (const count of [256, 257, 1025]) {
    const sent: TerminalInputEvent[] = [];
    const batches: number[] = [];
    let release!: () => void;
    const firstRequest = new Promise<void>(resolve => { release = resolve; });
    const queue = new TerminalInputQueue(async events => {
      batches.push(events.length);
      sent.push(...events);
      if (batches.length === 1) await firstRequest;
    }, error => { throw error; });
    const events: TerminalInputEvent[] = Array.from({ length: count }, (_, index) =>
      ({ kind: "key", key: "Character", text: String(index) }));
    queue.push(events);
    assert.equal(sent.length, 0);
    queue.start();
    queue.push([{ kind: "focusLost" }]);
    assert.equal(batches.length, 1);
    release();
    await new Promise(resolve => setImmediate(resolve));
    assert.ok(batches.every(size => size <= 256));
    assert.deepEqual(sent, [...events, { kind: "focusLost" }]);
    queue.dispose();
  }
  let calls = 0;
  let failures = 0;
  const failing = new TerminalInputQueue(async () => { calls += 1; throw new Error("adapter failed"); },
    () => { failures += 1; });
  failing.push(Array.from({ length: 257 }, () => ({ kind: "focusLost" })));
  failing.start();
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(calls, 1);
  assert.equal(failures, 1);
  failing.push([{ kind: "focusLost" }]);
  assert.equal(calls, 1);
}
