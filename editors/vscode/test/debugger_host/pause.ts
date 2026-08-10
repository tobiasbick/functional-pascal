/** Cooperative pause and disconnect host coverage. */

import assert from "node:assert/strict";

import * as vscode from "vscode";

import {
  closeAndRemoveSource,
  type DapMessage,
  startSession,
  waitFor,
  writeSource
} from "./support";

export async function verifyPauseAndDisconnect(
  workspaceRoot: string,
  received: DapMessage[],
  sent: DapMessage[]
): Promise<void> {
  const sourcePath = await writeSource(workspaceRoot, "pause", [
    "program DebuggerPause;",
    "",
    "begin",
    "  while true do",
    "  begin",
    "  end",
    "end.",
    ""
  ]);
  const marker = { received: received.length, sent: sent.length };
  let session: vscode.DebugSession | undefined;
  try {
    session = await startSession({
      type: "fpas",
      request: "launch",
      name: "FPAS debugger V1 pause",
      program: sourcePath,
      cwd: workspaceRoot,
      stopOnEntry: false
    });
    await waitFor(
      () =>
        received
          .slice(marker.received)
          .some((message) => message.command === "configurationDone"),
      "running debuggee"
    );
    await vscode.commands.executeCommand("workbench.action.debug.pause");
    await waitFor(
      () =>
        sent.slice(marker.sent).some(
          (message) =>
            message.event === "stopped" && message.body?.reason === "pause"
        ),
      "pause stop"
    );
    assert.ok(
      received.slice(marker.received).some((message) => message.command === "pause")
    );
    await vscode.debug.stopDebugging(session);
    session = undefined;
    await waitFor(
      () =>
        received
          .slice(marker.received)
          .some((message) => message.command === "disconnect"),
      "disconnect request"
    );
  } finally {
    if (session) await vscode.debug.stopDebugging(session);
    await closeAndRemoveSource(sourcePath);
  }
}
