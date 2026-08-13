/** Real Extension Host coverage for completing the active callee. */

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
    "function Compute(Value: integer): integer;",
    "begin",
    "  var Offset: integer := 1;",
    "  return Value + Offset",
    "end;",
    "",
    "procedure Announce(Message: string);",
    "begin",
    "  WriteLn(Message)",
    "end;",
    "",
    "begin",
    "  var Answer: integer := Compute(41);",
    "  Announce('skip me');",
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
    const frameId = await waitUntilFrame(
      session as vscode.DebugSession,
      "compute",
      () => sent.slice(marker.sent)
    );
    await vscode.commands.executeCommand(FORCE_RETURN_COMMAND, {
      frameId,
      expression: "99"
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
    assert.equal(answer.value, "99");

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
    assert.equal(output, "skip me\n99\n", `continuation observed the forced result: ${JSON.stringify(output)}`);
    assert.ok(
      received.slice(marker.received).some((message) => message.command === "fpas/forceReturn"),
      "Extension Host forwards forced return"
    );
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
