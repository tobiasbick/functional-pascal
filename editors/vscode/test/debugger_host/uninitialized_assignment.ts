/** Real Extension Host coverage for initializing uninitialized mutable bindings. */

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

export async function verifyUninitializedAssignment(
  workspaceRoot: string,
  received: DapMessage[],
  sent: DapMessage[]
): Promise<void> {
  const lines = [
    "program DebuggerUninitializedAssignment;",
    "",
    "uses Std.Console;",
    "",
    "mutable var",
    "  GlobalCount: integer := 99;",
    "",
    "begin",
    "  mutable var Count: integer := 1;",
    "  mutable var Flag: boolean := false;",
    "  var Frozen: integer := 2;",
    "  WriteLn(Count);",
    "  WriteLn(GlobalCount);",
    "  if Flag then",
    "  begin",
    "    WriteLn(1)",
    "  end",
    "  else",
    "  begin",
    "    WriteLn(0)",
    "  end;",
    "  WriteLn(Frozen)",
    "end.",
    ""
  ];
  const sourcePath = await writeSource(workspaceRoot, "uninitialized-assignment", lines);
  const marker = { received: received.length, sent: sent.length };
  let session: vscode.DebugSession | undefined;
  try {
    session = await startSession({
      type: "fpas",
      request: "launch",
      name: "FPAS debugger uninitialized assignment",
      program: sourcePath,
      cwd: workspaceRoot,
      stopOnEntry: true
    });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "stopped"),
      "uninitialized-assignment entry"
    );
    await waitUntilUninitialized(session as vscode.DebugSession, "Count");

    await setLocal(session as vscode.DebugSession, "Count", "30", "30");
    await setExpression(session as vscode.DebugSession, "Flag", "true", "true");
    await setExpression(session as vscode.DebugSession, "GlobalCount", "8", "8");

    const count = await namedVariable(session as vscode.DebugSession, "Locals", "Count");
    assert.equal(count.value, "30");
    await waitFor(
      () => eventCount(sent.slice(marker.sent), "invalidated") >= 3,
      "uninitialized assignment invalidation events"
    );
    const invalidations = eventCount(sent.slice(marker.sent), "invalidated");
    await assert.rejects(
      async () => setLocal(session as vscode.DebugSession, "Frozen", "9", "9"),
      /not mutable/i
    );
    assert.equal(
      eventCount(sent.slice(marker.sent), "invalidated"),
      invalidations,
      "rejected uninitialized assignments emit no invalidation"
    );

    await session.customRequest("continue", { threadId: 1 });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "terminated"),
      "uninitialized-assignment termination"
    );
    const output = sent
      .slice(marker.sent)
      .filter((message) => message.event === "output")
      .map((message) => String(message.body?.output ?? ""))
      .join("");
    assert.equal(
      output,
      "30\n8\n1\n2\n",
      `exact source initializer suppression preserves debugger values: ${JSON.stringify(
        sent.slice(marker.sent).filter((message) => message.event === "output")
      )}`
    );
    assert.ok(
      received.slice(marker.received).some((message) => message.command === "setVariable"),
      "Extension Host forwards handle-based initialization"
    );
    assert.ok(
      received.slice(marker.received).some((message) => message.command === "setExpression"),
      "Extension Host forwards textual initialization"
    );
  } finally {
    if (session) await vscode.debug.stopDebugging(session);
    await closeAndRemoveSource(sourcePath);
  }
}

async function waitUntilUninitialized(
  session: vscode.DebugSession,
  name: string
): Promise<void> {
  for (let attempt = 0; attempt < 64; attempt += 1) {
    const variable = await namedVariable(session, "Locals", name).catch(() => undefined);
    if (variable?.value === "<uninitialized>") return;
    if (variable && variable.value !== "<uninitialized>") {
      throw new Error(`${name} already initialized as ${variable.value}`);
    }
    await session.customRequest("stepIn", { threadId: 1 });
  }
  throw new Error(`${name} never became visible uninitialized`);
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
