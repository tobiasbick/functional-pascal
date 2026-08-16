/** Real Extension Host coverage for the standard DAP frame-restart flow. */

import assert from "node:assert/strict";

import * as vscode from "vscode";

import {
  closeAndRemoveSource,
  type DapMessage,
  startSession,
  waitFor,
  waitForStoppedReady,
  writeSource
} from "./support";

/** Verify advertised frame restart, invalidation, and explicit continuation. */
export async function verifyFrameRestart(
  workspaceRoot: string,
  received: DapMessage[],
  sent: DapMessage[]
): Promise<void> {
  const lines = [
    "program DebuggerFrameRestart;",
    "",
    "uses Std.Console;",
    "",
    "function Branch(Value: integer): integer;",
    "begin",
    "  mutable var Local: integer := Value + 10;",
    "  WriteLn('effect');",
    "  return Local",
    "end;",
    "",
    "begin",
    "  WriteLn(Branch(1))",
    "end.",
    ""
  ];
  const sourcePath = await writeSource(workspaceRoot, "frame-restart", lines);
  const breakpointLine = lines.indexOf("  return Local");
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
      name: "FPAS frame restart",
      program: sourcePath,
      cwd: workspaceRoot,
      stopOnEntry: false
    });
    await waitForStoppedReady(() => sent.slice(marker.sent), 1, "frame-restart stop");
    const initialize = sent
      .slice(marker.sent)
      .find((message) => message.type === "response" && message.command === "initialize");
    assert.equal(initialize?.body?.supportsRestartFrame, true);
    const stack = await session.customRequest("stackTrace", {
      threadId: 1,
      startFrame: 0,
      levels: 8
    }) as { stackFrames: Array<{ id: number; name: string }> };
    const selected = stack.stackFrames[0];
    assert.equal(selected?.name, "branch");
    const invalidationsBefore = sent
      .slice(marker.sent)
      .filter((message) => message.event === "invalidated").length;
    const outputBefore = output(sent.slice(marker.sent));
    await session.customRequest("restartFrame", { frameId: selected.id });
    await waitFor(
      () => sent
        .slice(marker.sent)
        .filter((message) => message.event === "invalidated").length > invalidationsBefore,
      "frame-restart invalidation"
    );
    assert.equal(output(sent.slice(marker.sent)), outputBefore, "restart dispatches no code");
    const invalidation = sent
      .slice(marker.sent)
      .reverse()
      .find((message) => message.event === "invalidated");
    assert.deepEqual(invalidation?.body?.areas, ["stacks", "variables"]);
    const forwarded = received
      .slice(marker.received)
      .find((message) => message.command === "restartFrame");
    assert.equal(forwarded?.arguments?.frameId, selected.id);
    const refreshed = await session.customRequest("stackTrace", {
      threadId: 1,
      startFrame: 0,
      levels: 1
    }) as { stackFrames: Array<{ id: number; name: string }> };
    assert.equal(refreshed.stackFrames[0]?.name, "branch");
    assert.notEqual(refreshed.stackFrames[0]?.id, selected.id, "old frame ID expires");

    const breakpointRequests = received
      .slice(marker.received)
      .filter((message) => message.command === "setBreakpoints").length;
    vscode.debug.removeBreakpoints([breakpoint]);
    await waitFor(
      () => received
        .slice(marker.received)
        .filter((message) => message.command === "setBreakpoints").length > breakpointRequests,
      "frame-restart breakpoint removal"
    );
    await session.customRequest("continue", { threadId: 1 });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "terminated"),
      "frame-restart termination"
    );
    assert.equal(output(sent.slice(marker.sent)), "effect\neffect\n11\n");
  } finally {
    vscode.debug.removeBreakpoints([breakpoint]);
    if (session) await vscode.debug.stopDebugging(session);
    await closeAndRemoveSource(sourcePath);
  }
}

function output(messages: readonly DapMessage[]): string {
  return messages
    .filter((message) => message.event === "output")
    .map((message) => String(message.body?.output ?? ""))
    .join("");
}
