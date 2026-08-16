/** Real Extension Host coverage for retained completed task-result replacement. */

import assert from "node:assert/strict";

import * as vscode from "vscode";

import { REPLACE_TASK_RESULT_COMMAND } from "../../src/debugger/taskResultCommand";
import {
  closeAndRemoveSource,
  type DapMessage,
  startSession,
  waitFor,
  waitForStoppedReady,
  writeSource
} from "./support";

/** Verify the editor command forwards a typed replacement before task consumption. */
export async function verifyTaskResultReplacement(
  workspaceRoot: string,
  received: DapMessage[],
  sent: DapMessage[]
): Promise<void> {
  const registered = await vscode.commands.getCommands(true);
  assert.ok(
    registered.includes(REPLACE_TASK_RESULT_COMMAND),
    "completed task-result command is registered"
  );
  const lines = [
    "program DebuggerTaskResult;",
    "",
    "uses Std.Console, Std.Task;",
    "",
    "function Work(): integer;",
    "begin",
    "  return 7",
    "end;",
    "",
    "begin",
    "  var Pending: task := go Work();",
    "  WriteLn(Wait(Pending))",
    "end.",
    ""
  ];
  const sourcePath = await writeSource(workspaceRoot, "task-result-replacement", lines);
  const breakpointLine = lines.indexOf("  return 7");
  const breakpoint = new vscode.SourceBreakpoint(
    new vscode.Location(vscode.Uri.file(sourcePath), new vscode.Position(breakpointLine, 2))
  );
  const marker = { received: received.length, sent: sent.length };
  let session: vscode.DebugSession | undefined;
  vscode.debug.addBreakpoints([breakpoint]);
  try {
    session = await startSession({
      type: "fpas",
      request: "launch",
      name: "FPAS completed task result",
      program: sourcePath,
      cwd: workspaceRoot,
      stopOnEntry: false
    });
    await waitForStoppedReady(
      () => sent.slice(marker.sent),
      1,
      "completed task-result child stop"
    );
    const stop = sent
      .slice(marker.sent)
      .find((message) => message.event === "stopped");
    const childThread = stop?.body?.threadId;
    assert.equal(typeof childThread, "number", "child stop exposes a DAP thread ID");
    const childStack = await session.customRequest("stackTrace", {
      threadId: childThread,
      startFrame: 0,
      levels: 1
    }) as { stackFrames: Array<{ id: number }> };
    const childFrame = childStack.stackFrames[0];
    assert.ok(childFrame, "child entry frame is inspectable");
    await session.customRequest("fpas/forceReturn", {
      frameId: childFrame.id,
      expression: "7"
    });

    const rootStack = await session.customRequest("stackTrace", {
      threadId: 1,
      startFrame: 0,
      levels: 1
    }) as { stackFrames: Array<{ id: number }> };
    const rootFrame = rootStack.stackFrames[0];
    assert.ok(rootFrame, "waiting root frame remains inspectable");
    const invalidationsBefore = sent
      .slice(marker.sent)
      .filter((message) => message.event === "invalidated").length;
    await vscode.commands.executeCommand(REPLACE_TASK_RESULT_COMMAND, {
      taskId: 1,
      frameId: rootFrame.id,
      expression: "9"
    });
    await waitFor(
      () => sent
        .slice(marker.sent)
        .filter((message) => message.event === "invalidated").length > invalidationsBefore,
      "completed task-result invalidation"
    );
    const invalidation = sent
      .slice(marker.sent)
      .reverse()
      .find((message) => message.event === "invalidated");
    assert.deepEqual(invalidation?.body?.areas, ["variables"]);
    const forwarded = received
      .slice(marker.received)
      .find((message) => message.command === "fpas/replaceTaskResult");
    assert.equal(forwarded?.arguments?.taskId, 1);
    assert.equal(forwarded?.arguments?.frameId, rootFrame.id);
    assert.equal(forwarded?.arguments?.expression, "9");

    await session.customRequest("continue", { threadId: 1 });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "terminated"),
      "completed task-result termination"
    );
    assert.ok(
      sent.slice(marker.sent).some(
        (message) => message.event === "output" && message.body?.output === "9\n"
      ),
      "the root task consumes the replacement result"
    );
  } finally {
    vscode.debug.removeBreakpoints([breakpoint]);
    if (session) await vscode.debug.stopDebugging(session);
    await closeAndRemoveSource(sourcePath);
  }
}
