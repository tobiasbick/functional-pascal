/** Conditional, hit-condition, and non-stopping logpoint host coverage. */

import assert from "node:assert/strict";

import * as vscode from "vscode";

import {
  closeAndRemoveSource,
  type DapMessage,
  startSession,
  waitFor,
  writeSource
} from "./support";

export async function verifyBreakpointPolicies(
  workspaceRoot: string,
  received: DapMessage[],
  sent: DapMessage[]
): Promise<void> {
  const lines = [
    "program DebuggerBreakpointPolicies;",
    "",
    "uses Std.Console;",
    "",
    "begin",
    "  mutable var Counter: integer := 0;",
    "  while Counter < 4 do",
    "  begin",
    "    Counter := Counter + 1;",
    "    WriteLn(Counter)",
    "  end",
    "end.",
    ""
  ];
  const sourcePath = await writeSource(workspaceRoot, "policies", lines);
  const line = lines.indexOf("    WriteLn(Counter)");
  try {
    await verifyExactHit(workspaceRoot, sourcePath, line, received, sent);
    await verifyLogpoint(workspaceRoot, sourcePath, line, received, sent);
    await verifyInvalidTemplate(workspaceRoot, sourcePath, line, sent);
  } finally {
    await closeAndRemoveSource(sourcePath);
  }
}

async function verifyExactHit(
  workspaceRoot: string,
  sourcePath: string,
  line: number,
  received: DapMessage[],
  sent: DapMessage[]
): Promise<void> {
  const breakpoint = new vscode.SourceBreakpoint(
    new vscode.Location(vscode.Uri.file(sourcePath), new vscode.Position(line, 4)),
    true,
    "Counter >= 2",
    "3"
  );
  const marker = { received: received.length, sent: sent.length };
  vscode.debug.addBreakpoints([breakpoint]);
  let session: vscode.DebugSession | undefined;
  try {
    session = await startSession({
      type: "fpas", request: "launch", name: "FPAS exact hit condition",
      program: sourcePath, cwd: workspaceRoot, stopOnEntry: false
    });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "stopped"),
      "exact third physical hit"
    );
    const stack = await session.customRequest("stackTrace", { threadId: 1 }) as {
      stackFrames: Array<{ id: number }>;
    };
    const evaluated = await session.customRequest("evaluate", {
      expression: "Counter", frameId: stack.stackFrames[0]?.id, context: "watch"
    }) as { result: string };
    assert.equal(evaluated.result, "3", "condition and exact hit count preserve physical hits");
    const outgoing = received.slice(marker.received).find(
      (message) => message.command === "setBreakpoints"
    );
    const configured = (outgoing?.arguments as { breakpoints?: unknown[] } | undefined)?.breakpoints;
    assert.ok(configured, "setBreakpoints reaches the adapter");
    await session.customRequest("continue", { threadId: 1 });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "terminated"),
      "hit-condition termination"
    );
  } finally {
    vscode.debug.removeBreakpoints([breakpoint]);
    if (session) await vscode.debug.stopDebugging(session);
  }
}

async function verifyLogpoint(
  workspaceRoot: string,
  sourcePath: string,
  line: number,
  received: DapMessage[],
  sent: DapMessage[]
): Promise<void> {
  const breakpoint = new vscode.SourceBreakpoint(
    new vscode.Location(vscode.Uri.file(sourcePath), new vscode.Position(line, 4)),
    true,
    undefined,
    undefined,
    "counter={Counter} {{ok}}"
  );
  const marker = { received: received.length, sent: sent.length };
  vscode.debug.addBreakpoints([breakpoint]);
  let session: vscode.DebugSession | undefined;
  try {
    session = await startSession({
      type: "fpas", request: "launch", name: "FPAS non-stopping logpoint",
      program: sourcePath, cwd: workspaceRoot, stopOnEntry: false
    });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "terminated"),
      "logpoint session termination"
    );
    assert.equal(
      sent.slice(marker.sent).filter((message) => message.event === "stopped").length,
      0,
      "logpoints stay invisible"
    );
    const output = sent.slice(marker.sent)
      .filter((message) => message.event === "output")
      .map((message) => String(message.body?.output ?? ""))
      .join("");
    for (const count of [1, 2, 3, 4]) assert.match(output, new RegExp(`counter=${count} \\{ok\\}`));
    const outgoing = received.slice(marker.received).find(
      (message) => message.command === "setBreakpoints"
    );
    assert.ok(JSON.stringify(outgoing).includes("counter={Counter} {{ok}}"));
  } finally {
    vscode.debug.removeBreakpoints([breakpoint]);
    if (session) await vscode.debug.stopDebugging(session);
  }
}

async function verifyInvalidTemplate(
  workspaceRoot: string,
  sourcePath: string,
  line: number,
  sent: DapMessage[]
): Promise<void> {
  const breakpoint = new vscode.SourceBreakpoint(
    new vscode.Location(vscode.Uri.file(sourcePath), new vscode.Position(line, 4)),
    true,
    undefined,
    undefined,
    "{}"
  );
  const marker = sent.length;
  vscode.debug.addBreakpoints([breakpoint]);
  let session: vscode.DebugSession | undefined;
  try {
    session = await startSession({
      type: "fpas", request: "launch", name: "FPAS invalid logpoint",
      program: sourcePath, cwd: workspaceRoot, stopOnEntry: true
    });
    await waitFor(
      () => sent.slice(marker).some((message) => message.command === "setBreakpoints"),
      "invalid logpoint verification"
    );
    const response = sent.slice(marker).find(
      (message) => message.type === "response" && message.command === "setBreakpoints"
    );
    assert.equal(
      (response?.body as { breakpoints?: Array<{ verified?: boolean }> } | undefined)
        ?.breakpoints?.[0]?.verified,
      false
    );
  } finally {
    vscode.debug.removeBreakpoints([breakpoint]);
    if (session) await vscode.debug.stopDebugging(session);
  }
}
