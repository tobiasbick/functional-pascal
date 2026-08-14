/** Real Extension Host coverage for completing a selected callee. */

import assert from "node:assert/strict";

import * as vscode from "vscode";

import { FORCE_RETURN_COMMAND } from "../../src/debugger/forcedReturnCommand";
import {
  closeAndRemoveSource,
  type DapMessage,
  eventCount,
  startSession,
  waitFor,
  waitForStoppedReady,
  writeSource
} from "./support";

interface DapVariable {
  readonly name: string;
  readonly value: string;
}

export async function verifyForcedReturn(
  workspaceRoot: string,
  received: DapMessage[],
  sent: DapMessage[]
): Promise<void> {
  const registered = await vscode.commands.getCommands(true);
  assert.ok(registered.includes(FORCE_RETURN_COMMAND), "force-return command is registered");

  const lines = [
    "program DebuggerForcedReturn;",
    "",
    "uses Std.Console;",
    "",
    "function Leaf(Value: integer): integer;",
    "begin",
    "  WriteLn('leaf');",
    "  return Value + 1",
    "end;",
    "",
    "function Branch(Value: integer): integer;",
    "begin",
    "  var Local: integer := Value + 10;",
    "  var Nested: integer := Leaf(Local);",
    "  WriteLn('branch');",
    "  return Nested",
    "end;",
    "",
    "begin",
    "  var Answer: integer := Branch(1);",
    "  WriteLn(Answer)",
    "end.",
    ""
  ];
  const sourcePath = await writeSource(workspaceRoot, "forced-return", lines);
  const marker = { received: received.length, sent: sent.length };
  let session: vscode.DebugSession | undefined;
  try {
    session = await startSession({
      type: "fpas",
      request: "launch",
      name: "FPAS debugger forced return",
      program: sourcePath,
      cwd: workspaceRoot,
      stopOnEntry: true
    });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "stopped"),
      "forced-return entry"
    );
    await waitUntilFrame(
      session as vscode.DebugSession,
      "leaf",
      () => sent.slice(marker.sent)
    );
    const stack = await (session as vscode.DebugSession).customRequest("stackTrace", {
      threadId: 1,
      startFrame: 0,
      levels: 8
    }) as { stackFrames: Array<{ id: number; name: string }> };
    const selected = stack.stackFrames.find((frame) => frame.name === "branch");
    assert.ok(selected, `older branch frame should be selectable: ${JSON.stringify(stack.stackFrames)}`);
    await vscode.commands.executeCommand(FORCE_RETURN_COMMAND, {
      frameId: selected.id,
      expression: "Local"
    });
    await waitFor(
      () => eventCount(sent.slice(marker.sent), "invalidated") >= 1,
      "forced-return invalidation"
    );
    const invalidation = sent
      .slice(marker.sent)
      .reverse()
      .find((message) => message.event === "invalidated");
    const areas = invalidation?.body?.areas as string[] | undefined;
    assert.deepEqual(areas, ["stacks", "variables"]);

    const answer = await namedVariable(session as vscode.DebugSession, "Locals", "Answer");
    assert.equal(answer.value, "11");

    await session.customRequest("continue", { threadId: 1 });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "terminated"),
      "forced-return termination"
    );
    const output = sent
      .slice(marker.sent)
      .filter((message) => message.event === "output")
      .map((message) => String(message.body?.output ?? ""))
      .join("");
    assert.equal(output, "11\n", `continuation skipped younger bodies: ${JSON.stringify(output)}`);
    const request = received
      .slice(marker.received)
      .find((message) => message.command === "fpas/forceReturn");
    assert.ok(request, "Extension Host forwards forced return");
    assert.equal(request.arguments?.frameId, selected.id, "command forwards the selected older frame");
    assert.equal(request.arguments?.expression, "Local");
  } finally {
    if (session) await vscode.debug.stopDebugging(session);
    await closeAndRemoveSource(sourcePath);
  }
}

async function waitUntilFrame(
  session: vscode.DebugSession,
  name: string,
  messages: () => readonly DapMessage[]
): Promise<number> {
  let expectedStops = eventCount(messages(), "stopped");
  for (let attempt = 0; attempt < 64; attempt += 1) {
    const stack = await session.customRequest("stackTrace", {
      threadId: 1,
      startFrame: 0,
      levels: 8
    }) as { stackFrames: Array<{ id: number; name: string }> };
    const current = stack.stackFrames[0];
    if (current?.name === name) return current.id;
    await session.customRequest("stepIn", { threadId: 1 });
    expectedStops += 1;
    await waitForStoppedReady(messages, expectedStops, `forced-return step into ${name}`);
  }
  throw new Error(`${name} never became the active frame`);
}

async function namedVariable(
  session: vscode.DebugSession,
  scopeName: string,
  name: string
): Promise<DapVariable> {
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
  const result = await session.customRequest("variables", {
    variablesReference: scope.variablesReference,
    start: 0,
    count: 100
  }) as { variables: DapVariable[] };
  const variable = result.variables.find((candidate) => candidate.name === name);
  assert.ok(variable, `${scopeName} contains ${name}`);
  return variable;
}
