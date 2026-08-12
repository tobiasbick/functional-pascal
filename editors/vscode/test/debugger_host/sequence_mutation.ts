/** Real Extension Host coverage for array and string structure custom requests. */

import assert from "node:assert/strict";

import * as vscode from "vscode";

import { SEQUENCE_COMMANDS } from "../../src/debugger/sequenceCommands";
import {
  closeAndRemoveSource,
  type DapMessage,
  eventCount,
  startSession,
  waitFor,
  writeSource
} from "./support";

/** Verify registration, stopped-session behavior, requests, errors, and continuation. */
export async function verifySequenceMutation(
  workspaceRoot: string,
  received: DapMessage[],
  sent: DapMessage[]
): Promise<void> {
  const registered = await vscode.commands.getCommands(true);
  for (const command of Object.values(SEQUENCE_COMMANDS)) {
    assert.ok(registered.includes(command), `sequence command ${command} is registered`);
  }
  await vscode.debug.stopDebugging();
  await waitFor(
    () => vscode.debug.activeDebugSession === undefined,
    "no active session before sequence command availability check"
  );
  const requestCount = (): number => received.filter(
    (message) => message.command?.startsWith("fpas/array")
      || message.command === "fpas/stringReplaceCharacter"
  ).length;
  const beforeUnavailable = requestCount();
  await vscode.commands.executeCommand(SEQUENCE_COMMANDS.insertArray, {
    frameId: 1,
    target: "Numbers",
    index: "0",
    value: "1"
  });
  assert.equal(
    requestCount(),
    beforeUnavailable,
    "sequence command sends no request without an active FPAS session"
  );

  const lines = [
    "program DebuggerSequenceMutation;",
    "",
    "uses Std.Console;",
    "",
    "begin",
    "  mutable var Numbers: array of integer := [1, 2, 3];",
    "  mutable var Text: string := 'A😀B';",
    "  var StopMarker: integer := 0;",
    "  WriteLn(Numbers[0]);",
    "  WriteLn(Numbers[1]);",
    "  WriteLn(Numbers[2]);",
    "  WriteLn(Text)",
    "end.",
    ""
  ];
  const sourcePath = await writeSource(workspaceRoot, "sequence-mutation", lines);
  const stopLine = lines.indexOf("  var StopMarker: integer := 0;");
  const breakpoint = new vscode.SourceBreakpoint(
    new vscode.Location(vscode.Uri.file(sourcePath), new vscode.Position(stopLine, 2))
  );
  const marker = { received: received.length, sent: sent.length };
  vscode.debug.addBreakpoints([breakpoint]);
  let session: vscode.DebugSession | undefined;

  try {
    session = await startSession({
      type: "fpas",
      request: "launch",
      name: "FPAS debugger sequence mutation",
      program: sourcePath,
      cwd: workspaceRoot,
      stopOnEntry: false
    });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "stopped"),
      "sequence-mutation breakpoint"
    );

    let frameId = await currentFrame(session);
    const beforeCancelledPrompt = requestCount();
    const cancelledPrompt = vscode.commands.executeCommand(SEQUENCE_COMMANDS.insertArray, {
      frameId
    });
    await new Promise((resolve) => setTimeout(resolve, 100));
    await vscode.commands.executeCommand("workbench.action.closeQuickOpen");
    await cancelledPrompt;
    assert.equal(
      requestCount(),
      beforeCancelledPrompt,
      "cancelled sequence prompt sends no custom request"
    );

    await vscode.commands.executeCommand(SEQUENCE_COMMANDS.insertArray, {
      frameId,
      target: "Numbers",
      index: "1",
      value: "9"
    });
    frameId = await currentFrame(session);
    await vscode.commands.executeCommand(SEQUENCE_COMMANDS.removeArray, {
      frameId,
      target: "Numbers",
      index: "2"
    });
    frameId = await currentFrame(session);
    await vscode.commands.executeCommand(SEQUENCE_COMMANDS.replaceStringCharacter, {
      frameId,
      target: "Text",
      index: "1",
      value: "'é'"
    });
    await waitFor(
      () => eventCount(sent.slice(marker.sent), "invalidated") >= 3,
      "sequence mutation invalidation events"
    );

    const invalidations = eventCount(sent.slice(marker.sent), "invalidated");
    frameId = await currentFrame(session);
    await assert.rejects(
      async () => session?.customRequest("fpas/arrayRemove", {
        frameId,
        target: "Numbers",
        index: "9"
      }),
      /outside/i
    );
    assert.equal(eventCount(sent.slice(marker.sent), "invalidated"), invalidations);

    await session.customRequest("continue", { threadId: 1 });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "terminated"),
      "sequence-mutation termination"
    );
    for (const output of ["1\n", "9\n", "3\n", "AéB\n"]) {
      assert.ok(
        sent.slice(marker.sent).some(
          (message) => message.event === "output" && message.body?.output === output
        ),
        `sequence continuation emits ${JSON.stringify(output)}`
      );
    }
    for (const command of [
      "fpas/arrayInsert",
      "fpas/arrayRemove",
      "fpas/stringReplaceCharacter"
    ]) {
      assert.ok(
        received.slice(marker.received).some((message) => message.command === command),
        `Extension Host forwards ${command}`
      );
    }
  } finally {
    vscode.debug.removeBreakpoints([breakpoint]);
    if (session) await vscode.debug.stopDebugging(session);
    await closeAndRemoveSource(sourcePath);
  }
}

async function currentFrame(session: vscode.DebugSession): Promise<number> {
  const stack = await session.customRequest("stackTrace", {
    threadId: 1,
    startFrame: 0,
    levels: 1
  }) as { stackFrames: Array<{ id: number }> };
  const frame = stack.stackFrames[0];
  assert.ok(frame, "stopped sequence session exposes a frame");
  return frame.id;
}
