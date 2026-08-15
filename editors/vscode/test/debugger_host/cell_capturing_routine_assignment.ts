/** Real Extension Host coverage for assigning cell-capturing named nested routines. */

import assert from "node:assert/strict";

import * as vscode from "vscode";

import {
  closeAndRemoveSource,
  type DapMessage,
  eventCount,
  startSession,
  waitFor,
  writeSource
} from "./support";

interface DapVariable {
  readonly name: string;
  readonly value: string;
  readonly variablesReference: number;
}

export async function verifyCellCapturingRoutineAssignment(
  workspaceRoot: string,
  received: DapMessage[],
  sent: DapMessage[]
): Promise<void> {
  const lines = [
    "program DebuggerCellCapturingRoutineAssignment;",
    "",
    "uses Std.Console;",
    "",
    "type",
    "  Handler = function(Value: integer): integer;",
    "",
    "function Identity(Value: integer): integer;",
    "begin",
    "  return Value",
    "end;",
    "",
    "function Mutating(): Handler;",
    "  function AddCell(Value: integer): integer;",
    "  begin",
    "    Cell := Cell + 1;",
    "    return Value + Cell",
    "  end;",
    "begin",
    "  mutable var Cell: integer := 1;",
    "  var Original: Handler := AddCell;",
    "  mutable var Current: Handler := Identity;",
    "  var CellStop: integer := 0;",
    "  Cell := Cell + 10;",
    "  WriteLn(Current(0));",
    "  WriteLn(Original(0));",
    "  return Current",
    "end;",
    "",
    "begin",
    "  var Output: Handler := Mutating();",
    "  WriteLn(Output(0))",
    "end.",
    ""
  ];
  const sourcePath = await writeSource(workspaceRoot, "cell-capturing-routine-assignment", lines);
  const cellStopLine = lines.findIndex((line) => line.includes("var CellStop: integer := 0"));
  assert.ok(cellStopLine >= 0, "compact program includes CellStop");
  const breakpoint = new vscode.SourceBreakpoint(
    new vscode.Location(vscode.Uri.file(sourcePath), new vscode.Position(cellStopLine, 0)),
    true
  );
  vscode.debug.addBreakpoints([breakpoint]);
  const marker = { received: received.length, sent: sent.length };
  let session: vscode.DebugSession | undefined;
  try {
    session = await startSession({
      type: "fpas",
      request: "launch",
      name: "FPAS debugger cell capturing routine assignment",
      program: sourcePath,
      cwd: workspaceRoot,
      stopOnEntry: false
    });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "stopped"),
      "cell-capturing-routine-assignment CellStop"
    );

    await setLocal(session as vscode.DebugSession, "Current", "AddCell", "<function mutating.addcell>");
    await setExpression(
      session as vscode.DebugSession,
      "Current",
      "Mutating.AddCell",
      "<function mutating.addcell>"
    );
    const current = await namedVariable(session as vscode.DebugSession, "Locals", "Current");
    assert.equal(current.value, "<function mutating.addcell>");
    await waitFor(
      () => eventCount(sent.slice(marker.sent), "invalidated") >= 1,
      "cell capturing routine assignment invalidation events"
    );
    const invalidations = eventCount(sent.slice(marker.sent), "invalidated");
    await assert.rejects(
      async () => setExpression(session as vscode.DebugSession, "Current", "MissingRoutine", "<function>"),
      /unknown|not a visible/i
    );
    assert.equal(
      eventCount(sent.slice(marker.sent), "invalidated"),
      invalidations,
      "rejected cell capturing routine assignments emit no invalidation"
    );

    await session.customRequest("continue", { threadId: 1 });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "terminated"),
      "cell-capturing-routine-assignment termination"
    );
    const output = sent
      .slice(marker.sent)
      .filter((message) => message.event === "output")
      .map((message) => String(message.body?.output ?? ""))
      .join("")
      .replaceAll("\r\n", "\n");
    assert.equal(
      output,
      "12\n13\n14\n",
      `continuation shared the mutable cell: ${JSON.stringify(output)}`
    );
    assert.ok(
      received.slice(marker.received).some((message) => message.command === "setVariable"),
      "Extension Host forwards Variables cell-capturing-routine assignment"
    );
    assert.ok(
      received.slice(marker.received).some((message) => message.command === "setExpression"),
      "Extension Host forwards Watch cell-capturing-routine assignment"
    );
  } finally {
    vscode.debug.removeBreakpoints([breakpoint]);
    if (session) await vscode.debug.stopDebugging(session);
    await closeAndRemoveSource(sourcePath);
  }
}

async function setLocal(
  session: vscode.DebugSession,
  name: string,
  value: string,
  expected: string
): Promise<void> {
  const reference = await scopeReference(session, "Locals");
  const result = await session.customRequest("setVariable", {
    variablesReference: reference,
    name,
    value
  }) as { value: string };
  assert.equal(result.value, expected);
}

async function setExpression(
  session: vscode.DebugSession,
  expression: string,
  value: string,
  expected: string
): Promise<void> {
  const stack = await session.customRequest("stackTrace", {
    threadId: 1,
    startFrame: 0,
    levels: 1
  }) as { stackFrames: Array<{ id: number }> };
  const result = await session.customRequest("setExpression", {
    frameId: stack.stackFrames[0]?.id,
    expression,
    value
  }) as { value: string };
  assert.equal(result.value, expected);
}

async function scopeReference(
  session: vscode.DebugSession,
  scopeName: string
): Promise<number> {
  const stack = await session.customRequest("stackTrace", {
    threadId: 1,
    startFrame: 0,
    levels: 1
  }) as { stackFrames: Array<{ id: number }> };
  const frame = stack.stackFrames[0];
  assert.ok(frame, "stopped session exposes a frame");
  const scopes = await session.customRequest("scopes", { frameId: frame.id }) as {
    scopes: Array<{ name: string; variablesReference: number }>;
  };
  const scope = scopes.scopes.find((candidate) => candidate.name === scopeName);
  assert.ok(scope, `stopped session exposes ${scopeName}`);
  return scope.variablesReference;
}

async function namedVariable(
  session: vscode.DebugSession,
  scopeName: string,
  name: string
): Promise<DapVariable> {
  const reference = await scopeReference(session, scopeName);
  const result = await session.customRequest("variables", {
    variablesReference: reference,
    start: 0,
    count: 100
  }) as { variables: DapVariable[] };
  const variable = result.variables.find((candidate) => candidate.name === name);
  assert.ok(variable, `${scopeName} contains ${name}`);
  return variable;
}
