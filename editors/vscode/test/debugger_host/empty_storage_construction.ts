/** Real Extension Host coverage for seeded empty-storage initialization. */

import assert from "node:assert/strict";

import * as vscode from "vscode";

import { INITIALIZE_STORAGE_COMMAND } from "../../src/debugger/storageInitializationCommand";
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

export async function verifyEmptyStorageConstruction(
  workspaceRoot: string,
  received: DapMessage[],
  sent: DapMessage[]
): Promise<void> {
  const registered = await vscode.commands.getCommands(true);
  assert.ok(
    registered.includes(INITIALIZE_STORAGE_COMMAND),
    "initialize-storage command is registered"
  );

  const lines = [
    "program DebuggerEmptyStorageConstruction;",
    "",
    "uses Std.Console;",
    "",
    "type",
    "  Point = record",
    "    X: integer;",
    "    Y: integer;",
    "  end;",
    "  Holder = record",
    "    Count: integer;",
    "    Nested: Point;",
    "  end;",
    "",
    "function MakeInitialState(): Holder;",
    "begin",
    "  return record",
    "    Count := 1;",
    "    Nested := record",
    "      X := 2;",
    "      Y := 3;",
    "    end;",
    "  end",
    "end;",
    "",
    "begin",
    "  mutable var State: Holder := MakeInitialState();",
    "  WriteLn(State.Count);",
    "  WriteLn(State.Nested.X)",
    "end.",
    ""
  ];
  const sourcePath = await writeSource(workspaceRoot, "empty-storage-construction", lines);
  const marker = { received: received.length, sent: sent.length };
  let session: vscode.DebugSession | undefined;
  try {
    session = await startSession({
      type: "fpas",
      request: "launch",
      name: "FPAS debugger empty-storage construction",
      program: sourcePath,
      cwd: workspaceRoot,
      stopOnEntry: true
    });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "stopped"),
      "empty-storage-construction entry"
    );
    await waitUntilUninitialized(session as vscode.DebugSession, "State");
    let frameId = await currentFrameId(session as vscode.DebugSession);

    await vscode.commands.executeCommand(INITIALIZE_STORAGE_COMMAND, {
      frameId,
      target: "State.Nested.X",
      initializer: "MakeInitialState()",
      expression: "42"
    });
    await waitFor(
      () => eventCount(sent.slice(marker.sent), "invalidated") >= 1,
      "empty-storage construction invalidation"
    );
    frameId = await currentFrameId(session as vscode.DebugSession);

    const state = await namedVariable(session as vscode.DebugSession, "Locals", "State");
    const nested = await childVariables(session as vscode.DebugSession, state.variablesReference);
    const point = named(nested, "Nested");
    const fields = await childVariables(session as vscode.DebugSession, point.variablesReference);
    assert.equal(named(fields, "X").value, "42");

    await assert.rejects(
      async () => vscode.commands.executeCommand(INITIALIZE_STORAGE_COMMAND, {
          frameId,
          target: "State.Count",
          initializer: "MakeInitialState()",
          expression: "1"
        }),
      /already initialized/i,
      "programmatic already-initialized roots reach the shared validator"
    );
    assert.equal(
      eventCount(sent.slice(marker.sent), "invalidated"),
      1,
      "rejected initialization does not invalidate variables"
    );

    await session.customRequest("continue", { threadId: 1 });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "terminated"),
      "empty-storage-construction termination"
    );
    const output = sent
      .slice(marker.sent)
      .filter((message) => message.event === "output")
      .map((message) => String(message.body?.output ?? ""))
      .join("");
    assert.equal(
      output,
      "1\n42\n",
      `exact source initializer suppression preserves debugger value: ${JSON.stringify(output)}`
    );
    assert.ok(
      received.slice(marker.received).some((message) => message.command === "fpas/initializeStorage"),
      "Extension Host forwards initialize storage"
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

async function currentFrameId(session: vscode.DebugSession): Promise<number> {
  const stack = await session.customRequest("stackTrace", {
    threadId: 1,
    startFrame: 0,
    levels: 1
  }) as { stackFrames: Array<{ id: number }> };
  const frame = stack.stackFrames[0];
  assert.ok(frame, "stopped session exposes a frame");
  return frame.id;
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
  return named(result.variables, name);
}

async function childVariables(
  session: vscode.DebugSession,
  variablesReference: number
): Promise<DapVariable[]> {
  const result = await session.customRequest("variables", {
    variablesReference,
    start: 0,
    count: 100
  }) as { variables: DapVariable[] };
  return result.variables;
}

function named(items: readonly DapVariable[], name: string): DapVariable {
  const variable = items.find((candidate) => candidate.name === name);
  assert.ok(variable, `variables contain ${name}`);
  return variable;
}
