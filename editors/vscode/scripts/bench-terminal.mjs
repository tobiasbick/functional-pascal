import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { performance } from "node:perf_hooks";

const require = createRequire(import.meta.url);
const { TerminalInputDecoder } = require("../out/src/debugger/terminal/input.js");
const rows = [];
for (const size of [4000, 8000, 16000]) {
  for (const mode of ["plain", "fragmented-paste"]) {
    const text = "x😀".repeat(size);
    const chunks = mode === "plain" ? [text] : [
      "\u001b[200~", ...text.match(/.{1,16}/gsu), "\u001b[201~"
    ];
    const samples = [];
    for (let pass = 0; pass < 4; pass += 1) {
      const decoder = new TerminalInputDecoder();
      const start = performance.now();
      const events = chunks.flatMap(chunk => decoder.feed(chunk));
      const elapsed = performance.now() - start;
      if (mode === "plain") assert.equal(events.length, size * 2);
      else assert.deepEqual(events, [{ kind: "paste", text }]);
      if (pass > 0) samples.push(elapsed);
    }
    samples.sort((a, b) => a - b);
    rows.push({ mode, size, median_ms: Number(samples[1].toFixed(3)) });
  }
}
console.log(JSON.stringify(rows, null, 2));
