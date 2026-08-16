/** Real Extension Host coverage for cancelling one spawned debug task. */

import assert from "node:assert/strict";

import * as vscode from "vscode";

import { CANCEL_TASK_COMMAND } from "../../src/debugger/taskControlCommand";
import {
  closeAndRemoveSource,
  type DapMessage,
  startSession,
  waitFor,
  waitForStoppedReady,
  writeSource
} from "./support";

/** Verify the editor command cancels one spawned task through DAP. */
export async function verifyTaskLifecycle(
  workspaceRoot: string,
  received: DapMessage[],
  sent: DapMessage[]
): Promise<void> {
  const registered = await vscode.commands.getCommands(true);
  assert.ok(registered.includes(CANCEL_TASK_COMMAND), "cancel-task command is registered");
  const lines = [
    "program DebuggerTaskLifecycle;",
    "",
    "uses Std.Console, Std.Task;",
    "",
    "function Work(): integer;",
    "begin",
    "  mutable var Value: integer := 40;",
    "  Value := Value + 2;",
    "  return Value",
    "end;",
    "",
    "begin",
    "  var Pending: task := go Work();",
    "  WriteLn(Wait(Pending))",
    "end.",
    ""
  ];
  const sourcePath = await writeSource(workspaceRoot, "task-lifecycle", lines);
  const breakpointLine = lines.indexOf("  Value := Value + 2;");
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
      name: "FPAS task lifecycle",
      program: sourcePath,
      cwd: workspaceRoot,
      stopOnEntry: false
    });
    await waitForStoppedReady(() => sent.slice(marker.sent), 1, "task-lifecycle child stop");
    const stop = sent
      .slice(marker.sent)
      .find((message) => message.event === "stopped");
    const childThread = stop?.body?.threadId;
    assert.equal(typeof childThread, "number", "child stop exposes a DAP thread ID");
    assert.notEqual(childThread, 1);

    await vscode.commands.executeCommand(CANCEL_TASK_COMMAND, { threadId: childThread });
    const cancelled = received
      .slice(marker.received)
      .find((message) => message.command === "fpas/cancelTask");
    assert.ok(cancelled, "cancel-task command sends fpas/cancelTask");
    await waitFor(
      () =>
        sent
          .slice(marker.sent)
          .some(
            (message) =>
              message.event === "thread" &&
              message.body?.reason === "exited" &&
              message.body?.threadId === childThread
          ),
      "task-lifecycle thread exit"
    );
  } finally {
    vscode.debug.removeBreakpoints([breakpoint]);
    if (session) await vscode.debug.stopDebugging(session);
    await closeAndRemoveSource(sourcePath);
  }
}
