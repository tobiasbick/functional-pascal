/** Real Extension Host coverage for dictionary structure custom requests. */

import assert from "node:assert/strict";

import * as vscode from "vscode";

import { DICTIONARY_COMMANDS } from "../../src/debugger/dictionaryCommands";
import {
  closeAndRemoveSource,
  type DapMessage,
  eventCount,
  startSession,
  waitFor,
  writeSource
} from "./support";

/** Verify registration, stopped-session behavior, requests, errors, and continuation. */
export async function verifyDictionaryMutation(
  workspaceRoot: string,
  received: DapMessage[],
  sent: DapMessage[]
): Promise<void> {
  const registered = await vscode.commands.getCommands(true);
  for (const command of Object.values(DICTIONARY_COMMANDS)) {
    assert.ok(registered.includes(command), `dictionary command ${command} is registered`);
  }
  await vscode.debug.stopDebugging();
  await waitFor(
    () => vscode.debug.activeDebugSession === undefined,
    "no active session before dictionary command availability check"
  );
  const dictionaryRequestCount = (): number => received.filter(
    (message) => message.command?.startsWith("fpas/dictionary")
  ).length;
  const beforeUnavailable = dictionaryRequestCount();
  await vscode.commands.executeCommand(DICTIONARY_COMMANDS.insert, {
    frameId: 1,
    target: "Scores",
    key: "'Ignored'",
    value: "1"
  });
  assert.equal(
    dictionaryRequestCount(),
    beforeUnavailable,
    "dictionary command sends no request without an active FPAS session"
  );

  const lines = [
    "program DebuggerDictionaryMutation;",
    "",
    "uses Std.Console;",
    "",
    "begin",
    "  mutable var Scores: dict of string to integer := ['Ada': 1, 'Grace': 2];",
    "  var StopMarker: integer := 0;",
    "  WriteLn(Scores['Hopper']);",
    "  WriteLn(Scores['Bob'])",
    "end.",
    ""
  ];
  const sourcePath = await writeSource(workspaceRoot, "dictionary-mutation", lines);
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
      name: "FPAS debugger dictionary mutation",
      program: sourcePath,
      cwd: workspaceRoot,
      stopOnEntry: false
    });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "stopped"),
      "dictionary-mutation breakpoint"
    );

    let frameId = await currentFrame(session);
    await vscode.commands.executeCommand(DICTIONARY_COMMANDS.insert, {
      frameId,
      target: "Scores",
      key: "'Bob'",
      value: "3"
    });

    frameId = await currentFrame(session);
    await vscode.commands.executeCommand(DICTIONARY_COMMANDS.remove, {
      frameId,
      target: "Scores",
      key: "'Ada'"
    });

    frameId = await currentFrame(session);
    await vscode.commands.executeCommand(DICTIONARY_COMMANDS.replaceKey, {
      frameId,
      target: "Scores",
      key: "'Grace'",
      newKey: "'Hopper'"
    });

    await waitFor(
      () => eventCount(sent.slice(marker.sent), "invalidated") >= 3,
      "dictionary mutation invalidation events"
    );
    const invalidations = eventCount(sent.slice(marker.sent), "invalidated");
    frameId = await currentFrame(session);
    await assert.rejects(
      async () => session?.customRequest("fpas/dictionaryRemove", {
        frameId,
        target: "Scores",
        key: "'Missing'"
      }),
      /does not exist/i
    );
    assert.equal(eventCount(sent.slice(marker.sent), "invalidated"), invalidations);

    await session.customRequest("continue", { threadId: 1 });
    try {
      await waitFor(
        () => sent.slice(marker.sent).some((message) => message.event === "terminated"),
        "dictionary-mutation termination"
      );
    } catch {
      assert.fail(`dictionary session did not terminate: ${JSON.stringify(sent.slice(marker.sent))}`);
    }
    assert.ok(
      sent.slice(marker.sent).some(
        (message) => message.event === "output" && message.body?.output === "2\n"
      )
    );
    assert.ok(
      sent.slice(marker.sent).some(
        (message) => message.event === "output" && message.body?.output === "3\n"
      )
    );
    for (const command of [
      "fpas/dictionaryInsert",
      "fpas/dictionaryRemove",
      "fpas/dictionaryReplaceKey"
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
  assert.ok(frame, "stopped dictionary session exposes a frame");
  return frame.id;
}
