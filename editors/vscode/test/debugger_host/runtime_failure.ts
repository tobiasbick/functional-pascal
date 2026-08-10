/** Runtime-failure inspection and unsupported-evaluation host coverage. */

import assert from "node:assert/strict";

import * as vscode from "vscode";

import {
  closeAndRemoveSource,
  type DapMessage,
  startSession,
  waitFor,
  writeSource
} from "./support";

interface DapStackFrame {
  readonly id: number;
}

export async function verifyRuntimeFailure(
  workspaceRoot: string,
  received: DapMessage[],
  sent: DapMessage[]
): Promise<void> {
  const sourcePath = await writeSource(workspaceRoot, "failure", [
    "program DebuggerFailure;",
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
      name: "FPAS debugger V1 runtime failure",
      program: sourcePath,
      cwd: workspaceRoot,
      stopOnEntry: false
    });
    await waitFor(
      () =>
        sent.slice(marker.sent).some(
          (message) =>
            message.event === "stopped" && message.body?.reason === "exception"
        ),
      "inspectable runtime failure"
    );
    const stack = await session.customRequest("stackTrace", {
      threadId: 1,
      startFrame: 0,
      levels: 64
    }) as { stackFrames: DapStackFrame[] };
    assert.ok(stack.stackFrames.length > 0, "runtime failure keeps stack inspection");
    await assert.rejects(
      async () =>
        session?.customRequest("evaluate", {
          expression: "Value",
          frameId: stack.stackFrames[0].id,
          context: "watch"
        }),
      /unsupported by FPAS debugger V1/u
    );
    await vscode.commands.executeCommand("workbench.action.debug.continue");
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "terminated"),
      "runtime failure termination"
    );
    assert.ok(
      sent.slice(marker.sent).some(
        (message) =>
          message.event === "output" && message.body?.category === "stderr"
      ),
      "runtime failure diagnostic reaches the Debug Console"
    );
  } finally {
    if (session) await vscode.debug.stopDebugging(session);
    await closeAndRemoveSource(sourcePath);
  }
}
