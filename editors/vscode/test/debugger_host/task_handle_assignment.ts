/** Real Extension Host coverage for assigning task handles. */

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

export async function verifyTaskHandleAssignment(
  workspaceRoot: string,
  received: DapMessage[],
  sent: DapMessage[]
): Promise<void> {
  const lines = [
    "program DebuggerTaskHandleAssignment;",
    "",
    "uses Std.Console, Std.Task;",
    "",
    "function Seven(): integer;",
    "begin",
    "  return 7",
    "end;",
    "",
    "function Nine(): integer;",
    "begin",
    "  return 9",
    "end;",
    "",
    "begin",
    "  var Pending: task := go Seven();",
    "  mutable var Current: task := go Nine();",
    "  var Frozen: task := go Seven();",
    "  mutable var StopMarker: integer := 0;",
    "  StopMarker := StopMarker + 1;",
    "  WriteLn(Wait(Current))",
    "end.",
    ""
  ];
  const sourcePath = await writeSource(workspaceRoot, "task-handle-assignment", lines);
  const marker = { received: received.length, sent: sent.length };
  let session: vscode.DebugSession | undefined;
  try {
    session = await startSession({
      type: "fpas",
      request: "launch",
      name: "FPAS debugger task-handle assignment",
      program: sourcePath,
      cwd: workspaceRoot,
      stopOnEntry: true
    });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "stopped"),
      "task-handle-assignment entry"
    );
    await waitUntilInitialized(session as vscode.DebugSession, "StopMarker");

    const pending = await namedVariable(session as vscode.DebugSession, "Locals", "Pending");
    assert.match(pending.value, /^<task \d+>$/);
    await setExpression(session as vscode.DebugSession, "Current", "Pending", pending.value);
    const current = await namedVariable(session as vscode.DebugSession, "Locals", "Current");
    assert.equal(current.value, pending.value);
    await waitFor(
      () => eventCount(sent.slice(marker.sent), "invalidated") >= 1,
      "task-handle assignment invalidation events"
    );
    const invalidations = eventCount(sent.slice(marker.sent), "invalidated");
    await assert.rejects(
      async () => setExpression(session as vscode.DebugSession, "Frozen", "Pending", pending.value),
      /not mutable/i
    );
    assert.equal(
      eventCount(sent.slice(marker.sent), "invalidated"),
      invalidations,
      "rejected task-handle assignments emit no invalidation"
    );

    await session.customRequest("continue", { threadId: 1 });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "terminated"),
      "task-handle-assignment termination"
    );
    const output = sent
      .slice(marker.sent)
      .filter((message) => message.event === "output")
      .map((message) => String(message.body?.output ?? ""))
      .join("");
    assert.equal(output, "7\n", `continuation waited on the copied handle: ${JSON.stringify(output)}`);
    assert.ok(
      received.slice(marker.received).some((message) => message.command === "setExpression"),
      "Extension Host forwards textual task-handle assignment"
    );
    assert.ok(
      !received.slice(marker.received).some((message) =>
        String(message.command ?? "").includes("forceReturn")
        || String(message.command ?? "").startsWith("fpas/task")
      ),
      "task-handle assignment adds no custom command"
    );
  } finally {
    if (session) await vscode.debug.stopDebugging(session);
    await closeAndRemoveSource(sourcePath);
  }
}

async function waitUntilInitialized(
  session: vscode.DebugSession,
  name: string
): Promise<void> {
  for (let attempt = 0; attempt < 64; attempt += 1) {
    const variable = await namedVariable(session, "Locals", name).catch(() => undefined);
    if (variable && variable.value !== "<uninitialized>") return;
    await session.customRequest("stepIn", { threadId: 1 });
  }
  throw new Error(`${name} never became initialized`);
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
