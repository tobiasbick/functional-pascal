/** Real Extension Host coverage for editing Variables-view values through DAP. */

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

export async function verifyVariableMutation(
  workspaceRoot: string,
  received: DapMessage[],
  sent: DapMessage[]
): Promise<void> {
  const lines = [
    "program DebuggerVariableMutation;",
    "",
    "uses Std.Console;",
    "",
    "type",
    "  Point = record",
    "    X: integer;",
    "    Y: integer;",
    "  end;",
    "",
    "mutable var",
    "  GlobalValue: integer := 5;",
    "",
    "begin",
    "  mutable var Scalar: integer := 1;",
    "  var Fixed: integer := 2;",
    "  mutable var Origin: Point := record",
    "    X := 3;",
    "    Y := 4;",
    "  end;",
    "  mutable var Items: array of integer := [6, 7];",
    "  mutable var Scores: dict of string to integer := ['Ada': 8];",
    "  var StopMarker: integer := Fixed;",
    "  WriteLn(Scalar + Origin.X + Items[1] + Scores['Ada'] + GlobalValue)",
    "end.",
    ""
  ];
  const sourcePath = await writeSource(workspaceRoot, "variable-mutation", lines);
  const stopLine = lines.indexOf("  var StopMarker: integer := Fixed;");
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
      name: "FPAS debugger variable mutation",
      program: sourcePath,
      cwd: workspaceRoot,
      stopOnEntry: false
    });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "stopped"),
      "variable-mutation breakpoint"
    );

    await setNamed(session, "Locals", "Scalar", "10");
    await setChild(session, "Locals", "Origin", "X", "20");
    await setChild(session, "Locals", "Items", "[1]", "30");
    await setChild(session, "Locals", "Scores", "[0].value", "40");
    await setNamed(session, "Globals", "GlobalValue", "50");

    const scalar = await namedVariable(session, "Locals", "Scalar");
    assert.equal(scalar.value, "10", "successful edit refreshes the Variables value");
    await assert.rejects(
      async () => setNamed(session as vscode.DebugSession, "Locals", "Fixed", "9"),
      /not mutable/i
    );
    assert.equal(
      (await namedVariable(session, "Locals", "Fixed")).value,
      "2",
      "rejected edit leaves the session and value stable"
    );
    await waitFor(
      () => eventCount(sent.slice(marker.sent), "invalidated") >= 5,
      "Variables-view invalidation events"
    );

    await session.customRequest("continue", { threadId: 1 });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "terminated"),
      "variable-mutation termination"
    );
    assert.ok(
      sent.slice(marker.sent).some(
        (message) => message.event === "output" && message.body?.output === "150\n"
      ),
      `continued debuggee observes every committed value: ${JSON.stringify(
        sent.slice(marker.sent).filter((message) => message.event === "output")
      )}`
    );
    assert.ok(
      received.slice(marker.received).filter((message) => message.command === "setVariable")
        .length >= 6,
      "real Extension Host forwards successful and rejected variable edits"
    );
  } finally {
    vscode.debug.removeBreakpoints([breakpoint]);
    if (session) await vscode.debug.stopDebugging(session);
    await closeAndRemoveSource(sourcePath);
  }
}

async function setNamed(
  session: vscode.DebugSession,
  scope: string,
  name: string,
  value: string
): Promise<void> {
  const reference = await scopeReference(session, scope);
  const result = await session.customRequest("setVariable", {
    variablesReference: reference,
    name,
    value
  }) as { value: string };
  assert.equal(result.value, value);
}

async function setChild(
  session: vscode.DebugSession,
  scope: string,
  parent: string,
  name: string,
  value: string
): Promise<void> {
  const variable = await namedVariable(session, scope, parent);
  assert.ok(variable.variablesReference > 0, `${parent} is expandable`);
  const result = await session.customRequest("setVariable", {
    variablesReference: variable.variablesReference,
    name,
    value
  }) as { value: string };
  assert.equal(result.value, value);
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

async function scopeReference(
  session: vscode.DebugSession,
  name: string
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
  const scope = scopes.scopes.find((candidate) => candidate.name === name);
  assert.ok(scope, `stopped session exposes ${name}`);
  return scope.variablesReference;
}
