/** Real Extension Host coverage for qualified single-payload variant transitions. */

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

export async function verifyVariantTransition(
  workspaceRoot: string,
  received: DapMessage[],
  sent: DapMessage[]
): Promise<void> {
  const lines = [
    "program DebuggerVariantTransition;",
    "",
    "uses Std.Console;",
    "",
    "type",
    "  Choice = enum",
    "    Empty;",
    "    Count(Value: integer);",
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
    "    Choice.Count(Value):",
    "    begin",
    "      return Value",
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
    "  mutable var Outcome: Result of integer, string := Ok(2);",
    "  mutable var Optional: Option of integer := None;",
    "  var Fixed: Choice := Choice.Count(9);",
    "  var StopMarker: integer := 0;",
    "  WriteLn(ChoiceValue(Selected));",
    "  case Outcome of",
    "    Ok(Value):",
    "    begin",
    "      WriteLn(Value)",
    "    end;",
    "    Error(Message):",
    "    begin",
    "      WriteLn(Message)",
    "    end",
    "  end;",
    "  case Optional of",
    "    Some(Value):",
    "    begin",
    "      WriteLn(Value)",
    "    end;",
    "    None:",
    "    begin",
    "      WriteLn(0)",
    "    end",
    "  end;",
    "  WriteLn(ChoiceValue(Fixed))",
    "end.",
    ""
  ];
  const sourcePath = await writeSource(workspaceRoot, "variant-transition", lines);
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
      name: "FPAS debugger variant transition",
      program: sourcePath,
      cwd: workspaceRoot,
      stopOnEntry: false
    });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "stopped"),
      "variant-transition breakpoint"
    );

    await setExpression(session, "Selected.Count.Value", "10", "Choice.Count");
    await setExpression(session, "Outcome.Error.value", "'fail'", "Error(...)");
    await setExpression(session, "Optional.Some.value", "8", "Some(...)");

    const selected = await namedVariable(session, "Locals", "Selected");
    assert.equal(selected.value, "Choice.Count");
    const selectedFields = await session.customRequest("variables", {
      variablesReference: selected.variablesReference,
      start: 0,
      count: 10
    }) as { variables: DapVariable[] };
    assert.equal(
      selectedFields.variables.find((field) => field.name === "Value")?.value,
      "10"
    );
    await waitFor(
      () => eventCount(sent.slice(marker.sent), "invalidated") >= 3,
      "variant transition invalidation events"
    );
    const invalidations = eventCount(sent.slice(marker.sent), "invalidated");
    await assert.rejects(
      async () => setExpression(session as vscode.DebugSession, "Selected.Pair.Left", "1", "1"),
      /unsupported|path|constructor|Pair/i
    );
    await assert.rejects(
      async () => setExpression(session as vscode.DebugSession, "Fixed.Count.Value", "1", "1"),
      /not mutable/i
    );
    assert.equal(
      eventCount(sent.slice(marker.sent), "invalidated"),
      invalidations,
      "rejected variant transitions emit no invalidation"
    );

    await session.customRequest("continue", { threadId: 1 });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "terminated"),
      "variant-transition termination"
    );
    const output = sent
      .slice(marker.sent)
      .filter((message) => message.event === "output")
      .map((message) => String(message.body?.output ?? ""))
      .join("");
    assert.equal(
      output,
      "10\nfail\n8\n9\n",
      `continued debuggee observes variant transitions: ${JSON.stringify(
        sent.slice(marker.sent).filter((message) => message.event === "output")
      )}`
    );
    assert.ok(
      received.slice(marker.received).some((message) => message.command === "setExpression"),
      "Extension Host forwards textual variant transitions"
    );
  } finally {
    vscode.debug.removeBreakpoints([breakpoint]);
    if (session) await vscode.debug.stopDebugging(session);
    await closeAndRemoveSource(sourcePath);
  }
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
