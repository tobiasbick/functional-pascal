import assert from "node:assert/strict";
import test from "node:test";

import { debugAdapterArguments, unsupportedDebugRequestReason } from "../src/debugger/adapter";

test("debug adapter arguments preserve target, source root, and program args", () => {
  assert.deepEqual(
    debugAdapterArguments(
      {
        type: "fpas",
        request: "launch",
        name: "test",
        program: "app.fpascp",
        sourceRoot: "sources",
        args: ["input.txt", "verbose"]
      },
      "standard-library"
    ),
    [
      "debug",
      "--std-lib",
      "standard-library",
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

test("debug adapter rejects attach instead of launching", () => {
  assert.equal(
    unsupportedDebugRequestReason("attach"),
    "Functional Pascal debugging does not support attach; use a launch configuration."
  );
  assert.equal(unsupportedDebugRequestReason("launch"), undefined);
});
