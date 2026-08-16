/** Real Extension Host coverage for per-task pause and resume holds. */

import assert from "node:assert/strict";

import * as vscode from "vscode";

import {
  PAUSE_TASK_COMMAND,
  RESUME_TASK_COMMAND
} from "../../src/debugger/taskControlCommand";
import {
  closeAndRemoveSource,
  type DapMessage,
  startSession,
  waitFor,
  waitForStoppedReady,
  writeSource
} from "./support";

/** Verify editor commands hold and release one spawned task through DAP. */
export async function verifyTaskControl(
  workspaceRoot: string,
  received: DapMessage[],
  sent: DapMessage[]
): Promise<void> {
  const registered = await vscode.commands.getCommands(true);
  assert.ok(registered.includes(PAUSE_TASK_COMMAND), "pause-task command is registered");
  assert.ok(registered.includes(RESUME_TASK_COMMAND), "resume-task command is registered");
  const lines = [
    "program DebuggerTaskControl;",
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
  const sourcePath = await writeSource(workspaceRoot, "task-control", lines);
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
      name: "FPAS task control",
      program: sourcePath,
      cwd: workspaceRoot,
      stopOnEntry: false
    });
    await waitForStoppedReady(() => sent.slice(marker.sent), 1, "task-control child stop");
    const stop = sent
      .slice(marker.sent)
      .find((message) => message.event === "stopped");
    const childThread = stop?.body?.threadId;
    assert.equal(typeof childThread, "number", "child stop exposes a DAP thread ID");
    assert.notEqual(childThread, 1);

    await vscode.commands.executeCommand(PAUSE_TASK_COMMAND, { threadId: childThread });
    const paused = received
      .slice(marker.received)
      .find((message) => message.command === "fpas/pauseTask");
    assert.ok(paused, "pause-task command sends fpas/pauseTask");
    const threads = await session.customRequest("threads") as {
      threads: Array<{ id: number; name: string }>;
    };
    assert.ok(
      threads.threads.some((thread) => thread.id === childThread && thread.name.includes("[paused]")),
      "paused task is marked in the Threads view"
    );

    await vscode.commands.executeCommand(RESUME_TASK_COMMAND, { threadId: childThread });
    const resumed = received
      .slice(marker.received)
      .find((message) => message.command === "fpas/resumeTask");
    assert.ok(resumed, "resume-task command sends fpas/resumeTask");

    vscode.debug.removeBreakpoints([breakpoint]);
    await session.customRequest("continue", { threadId: childThread });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "terminated"),
      "task-control termination"
    );
  } finally {
    vscode.debug.removeBreakpoints([breakpoint]);
    if (session) await vscode.debug.stopDebugging(session);
    await closeAndRemoveSource(sourcePath);
  }
}
