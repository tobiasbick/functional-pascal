/** Standard VS Code function-breakpoint host coverage. */

import assert from "node:assert/strict";

import * as vscode from "vscode";

import {
  closeAndRemoveSource,
  type DapMessage,
  startSession,
  waitFor,
  writeSource
} from "./support";

export async function verifyFunctionBreakpoints(
  workspaceRoot: string,
  received: DapMessage[],
  sent: DapMessage[]
): Promise<void> {
  const lines = [
    "program DebuggerFunctionBreakpoints;",
    "",
    "function Helper(Value: integer): integer;",
    "begin",
    "  return Value + 1",
    "end;",
    "",
    "begin",
    "  var First: integer := Helper(1);",
    "  var Second: integer := Helper(First)",
    "end.",
    ""
  ];
  const sourcePath = await writeSource(workspaceRoot, "function-breakpoints", lines);
  const breakpoint = new vscode.FunctionBreakpoint(
    "Helper",
    true,
    undefined,
    "2"
  );
  const marker = { received: received.length, sent: sent.length };
  vscode.debug.addBreakpoints([breakpoint]);
  let session: vscode.DebugSession | undefined;
  try {
    session = await startSession({
      type: "fpas",
      request: "launch",
      name: "FPAS function breakpoint",
      program: sourcePath,
      cwd: workspaceRoot,
      stopOnEntry: false
    });
    await waitFor(
      () => sent.slice(marker.sent).some(
        (message) => message.event === "stopped" && message.body?.reason === "breakpoint"
      ),
      "second Helper function hit"
    );
    const outgoing = received.slice(marker.received).find(
      (message) => message.command === "setFunctionBreakpoints"
    );
    assert.equal(
      (outgoing?.arguments as { breakpoints?: Array<{ name?: string; hitCondition?: string }> }
        | undefined)?.breakpoints?.[0]?.name,
      "Helper"
    );
    assert.equal(
      (outgoing?.arguments as { breakpoints?: Array<{ hitCondition?: string }> } | undefined)
        ?.breakpoints?.[0]?.hitCondition,
      "2"
    );
    const response = sent.slice(marker.sent).find(
      (message) => message.type === "response" && message.command === "setFunctionBreakpoints"
    );
    assert.equal(
      (response?.body as { breakpoints?: Array<{ verified?: boolean }> } | undefined)
        ?.breakpoints?.[0]?.verified,
      true
    );
    await session.customRequest("continue", { threadId: 1 });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "terminated"),
      "function-breakpoint termination"
    );
  } finally {
    vscode.debug.removeBreakpoints([breakpoint]);
    if (session) await vscode.debug.stopDebugging(session);
    await closeAndRemoveSource(sourcePath);
  }
}
