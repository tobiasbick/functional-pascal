/** Real Extension Host coverage for complete variant construction. */

import assert from "node:assert/strict";

import * as vscode from "vscode";

import { CONSTRUCT_VARIANT_COMMAND } from "../../src/debugger/variantConstructionCommand";
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

export async function verifyVariantConstruction(
  workspaceRoot: string,
  received: DapMessage[],
  sent: DapMessage[]
): Promise<void> {
  const registered = await vscode.commands.getCommands(true);
  assert.ok(
    registered.includes(CONSTRUCT_VARIANT_COMMAND),
    "construct-variant command is registered"
  );

  const lines = [
    "program DebuggerVariantConstruction;",
    "",
    "uses Std.Console;",
    "",
    "type",
    "  Choice = enum",
    "    Empty;",
    "    Pair(Left: integer; Right: integer);",
    "  end;",
    "",
    "function ChoiceValue(Item: Choice): integer;",
    "begin",
    "  case Item of",
    "    Choice.Empty:",
    "    begin",
    "      return 0",
    "    end;",
    "    Choice.Pair(Left, Right):",
    "    begin",
    "      return Left + Right",
    "    end",
    "  end",
    "end;",
    "",
    "begin",
    "  mutable var Selected: Choice := Choice.Empty;",
    "  var StopMarker: integer := 0;",
    "  WriteLn(ChoiceValue(Selected))",
    "end.",
    ""
  ];
  const sourcePath = await writeSource(workspaceRoot, "variant-construction", lines);
  const stopLine = lines.indexOf("  var StopMarker: integer := 0;");
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
      name: "FPAS debugger variant construction",
      program: sourcePath,
      cwd: workspaceRoot,
      stopOnEntry: false
    });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "stopped"),
      "variant-construction breakpoint"
    );
    let frameId = await currentFrameId(session as vscode.DebugSession);

    await vscode.commands.executeCommand(CONSTRUCT_VARIANT_COMMAND, {
      frameId,
      target: "Selected",
      variant: "Choice.Empty",
      fields: {}
    });
    await waitFor(
      () => eventCount(sent.slice(marker.sent), "invalidated") >= 1,
      "fieldless construction invalidation"
    );
    frameId = await currentFrameId(session as vscode.DebugSession);

    await assert.rejects(
      async () => vscode.commands.executeCommand(CONSTRUCT_VARIANT_COMMAND, {
          frameId,
          target: "Selected",
          variant: "Choice.Pair",
          fields: { Left: "1", Right: "2", Extra: "3" }
        }),
      /unknown field/i,
      "programmatic extra fields reach the shared exact-field validator"
    );
    assert.equal(
      eventCount(sent.slice(marker.sent), "invalidated"),
      1,
      "rejected construction does not invalidate variables"
    );

    await vscode.commands.executeCommand(CONSTRUCT_VARIANT_COMMAND, {
      frameId,
      target: "Selected",
      variant: "Choice.Pair",
      fields: { Left: "1", Right: "2" }
    });
    await waitFor(
      () => eventCount(sent.slice(marker.sent), "invalidated") >= 2,
      "multi-field construction invalidation"
    );

    const selected = await namedVariable(session as vscode.DebugSession, "Locals", "Selected");
    assert.equal(selected.value, "Choice.Pair");
    const fields = await childVariables(session as vscode.DebugSession, selected.variablesReference);
    assert.equal(named(fields, "Left").value, "1");
    assert.equal(named(fields, "Right").value, "2");

    await session.customRequest("continue", { threadId: 1 });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "terminated"),
      "variant-construction termination"
    );
    const output = sent
      .slice(marker.sent)
      .filter((message) => message.event === "output")
      .map((message) => String(message.body?.output ?? ""))
      .join("");
    assert.equal(output, "3\n", `continuation observed the constructed pair: ${JSON.stringify(output)}`);
    assert.ok(
      received.slice(marker.received).some((message) => message.command === "fpas/variantDescribe"),
      "Extension Host forwards variant describe"
    );
    assert.ok(
      received.slice(marker.received).some((message) => message.command === "fpas/variantConstruct"),
      "Extension Host forwards variant construct"
    );
  } finally {
    vscode.debug.removeBreakpoints([breakpoint]);
    if (session) await vscode.debug.stopDebugging(session);
    await closeAndRemoveSource(sourcePath);
  }
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
