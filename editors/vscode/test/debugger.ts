import assert from "node:assert/strict";
import test from "node:test";

import { debugAdapterArguments } from "../src/debugger/adapter";

test("debug adapter arguments preserve target, source root, and program args", () => {
  assert.deepEqual(
    debugAdapterArguments({
      type: "fpas",
      request: "launch",
      name: "test",
      program: "app.fpascp",
      sourceRoot: "sources",
      args: ["input.txt", "verbose"]
    }),
    [
      "debug",
      "app.fpascp",
      "--protocol",
      "dap",
      "--source-root",
      "sources",
      "--",
      "input.txt",
      "verbose"
    ]
  );
});
