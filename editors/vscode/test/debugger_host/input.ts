/** Real Extension Host coverage for queued debuggee ReadLn input. */

import assert from "node:assert/strict";

import * as vscode from "vscode";

import {
  CANCEL_INPUT_COMMAND,
  SEND_INPUT_COMMAND,
  SIGNAL_INPUT_EOF_COMMAND
} from "../../src/debugger/inputCommand";
import {
  closeAndRemoveSource,
  type DapMessage,
  startSession,
  waitFor,
  waitForStoppedReady,
  writeSource
} from "./support";

/** Verify ordered program input through the editor command, not Debug Console. */
export async function verifyDebuggerInput(
  workspaceRoot: string,
  received: DapMessage[],
  sent: DapMessage[]
): Promise<void> {
  const registered = await vscode.commands.getCommands(true);
  assert.ok(registered.includes(SEND_INPUT_COMMAND), "send-input command is registered");
  assert.ok(
    registered.includes(SIGNAL_INPUT_EOF_COMMAND),
    "signal-input-eof command is registered"
  );
  assert.ok(registered.includes(CANCEL_INPUT_COMMAND), "cancel-input command is registered");
  const lines = [
    "program DebuggerInput;",
    "",
    "uses Std.Console;",
    "",
    "begin",
    "  WriteLn(ReadLn());",
    "  WriteLn(ReadLn())",
    "end.",
    ""
  ];
  const sourcePath = await writeSource(workspaceRoot, "input", lines);
  const marker = { received: received.length, sent: sent.length };
  let session: vscode.DebugSession | undefined;
  try {
    session = await startSession({
      type: "fpas",
      request: "launch",
      name: "FPAS debugger input",
      program: sourcePath,
      cwd: workspaceRoot,
      stopOnEntry: true
    });
    await waitForStoppedReady(() => sent.slice(marker.sent), 1, "input entry stop");

    await vscode.commands.executeCommand(SEND_INPUT_COMMAND, { text: "one" });
    await vscode.commands.executeCommand(SEND_INPUT_COMMAND, { text: "two" });
    assert.equal(
      received.slice(marker.received).filter((message) => message.command === "fpas/input")
        .length,
      2,
      "send-input command forwards fpas/input"
    );

    const evaluated = await session.customRequest("evaluate", {
      expression: "1",
      context: "repl"
    }) as { result: string };
    assert.equal(evaluated.result, "1");
    assert.equal(
      received.slice(marker.received).filter((message) => message.command === "fpas/input")
        .length,
      2,
      "Debug Console evaluation does not inject debuggee stdin"
    );

    await session.customRequest("continue", { threadId: 1 });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "terminated"),
      "input termination"
    );
    const output = sent
      .slice(marker.sent)
      .filter((message) => message.event === "output")
      .map((message) => String(message.body?.output ?? ""))
      .join("");
    assert.equal(output, "one\ntwo\n", `queued ReadLn output: ${JSON.stringify(output)}`);
  } finally {
    if (session) await vscode.debug.stopDebugging(session);
    await closeAndRemoveSource(sourcePath);
  }
}
