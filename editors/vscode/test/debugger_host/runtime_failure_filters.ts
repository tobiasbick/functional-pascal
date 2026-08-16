/** VS Code exception-breakpoint filtering and failed-exit coverage. */

import assert from "node:assert/strict";

import * as vscode from "vscode";

import {
  closeAndRemoveSource,
  type DapMessage,
  startSession,
  waitFor,
  writeSource
} from "./support";

export async function verifyRuntimeFailureFilters(
  workspaceRoot: string,
  received: DapMessage[],
  sent: DapMessage[]
): Promise<void> {
  const sourcePath = await writeSource(workspaceRoot, "failure-filter", [
    "program DebuggerFailureFilter;",
    "",
    "begin",
    "  var Zero: integer := 0;",
    "  var Value: integer := 1 div Zero",
    "end.",
    ""
  ]);
  const marker = { received: received.length, sent: sent.length };
  let session: vscode.DebugSession | undefined;
  try {
    session = await startSession({
      type: "fpas",
      request: "launch",
      name: "FPAS runtime failure filter",
      program: sourcePath,
      cwd: workspaceRoot,
      stopOnEntry: true
    });
    await waitFor(
      () => sent.slice(marker.sent).some(
        (message) => message.event === "stopped" && message.body?.reason === "entry"
      ),
      "runtime-filter entry stop"
    );
    const configurationMarker = received.length;
    const response = await session.customRequest("setExceptionBreakpoints", {
      filters: ["F4010"]
    }) as Record<string, never>;
    assert.deepEqual(response, {});
    const configured = received.slice(configurationMarker).find(
      (message) => message.command === "setExceptionBreakpoints"
    );
    assert.deepEqual(
      (configured?.arguments as { filters?: string[] } | undefined)?.filters,
      ["F4010"]
    );

    const continuation = sent.length;
    await session.customRequest("continue", { threadId: 1 });
    await waitFor(
      () => sent.slice(continuation).some((message) => message.event === "terminated"),
      "filtered runtime failure termination"
    );
    const completion = sent.slice(continuation);
    assert.equal(
      completion.some((message) => message.event === "stopped"),
      false,
      "a nonmatching runtime code does not stop"
    );
    assert.ok(
      completion.some(
        (message) => message.event === "output" && message.body?.category === "stderr"
      ),
      "the filtered failure remains visible in the Debug Console"
    );
    assert.ok(
      completion.some(
        (message) => message.event === "exited" && message.body?.exitCode === 1
      ),
      "the filtered failure preserves a nonzero exit"
    );
  } finally {
    if (session) await vscode.debug.stopDebugging(session);
    await closeAndRemoveSource(sourcePath);
  }
}
