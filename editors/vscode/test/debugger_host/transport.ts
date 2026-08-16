/** Real Extension Host coverage for protocol-versus-debuggee I/O. */

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

/** Verify structured output without a second console runtime. */
export async function verifyDebuggerTransport(
  workspaceRoot: string,
  received: DapMessage[],
  sent: DapMessage[]
): Promise<void> {
  const lines = [
    "program DebuggerTransport;",
    "",
    "uses Std.Console;",
    "",
    "begin",
    "  WriteLn('hello-raw')",
    "end.",
    ""
  ];
  const sourcePath = await writeSource(workspaceRoot, "transport", lines);
  const marker = { received: received.length, sent: sent.length };
  let session: vscode.DebugSession | undefined;
  try {
    session = await startSession({
      type: "fpas",
      request: "launch",
      name: "FPAS debugger transport",
      program: sourcePath,
      cwd: workspaceRoot,
      stopOnEntry: true
    });
    await waitForStoppedReady(() => sent.slice(marker.sent), 1, "transport entry stop");

    const evaluated = await session.customRequest("evaluate", {
      expression: "1",
      context: "repl"
    }) as { result: string };
    assert.equal(evaluated.result, "1");
    assert.equal(
      received.slice(marker.received).filter((message) => message.command === "fpas/input").length,
      0,
      "Debug Console evaluation does not inject debuggee stdin"
    );

    await session.customRequest("continue", { threadId: 1 });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "terminated"),
      "transport termination"
    );
    const output = sent
      .slice(marker.sent)
      .filter((message) => message.event === "output")
      .map((message) => String(message.body?.output ?? ""))
      .join("");
    assert.equal(output, "hello-raw\n", `structured DAP output: ${JSON.stringify(output)}`);
  } finally {
    if (session) await vscode.debug.stopDebugging(session);
    await closeAndRemoveSource(sourcePath);
  }
}
