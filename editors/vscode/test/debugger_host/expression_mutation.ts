/** Real Extension Host coverage for standard DAP textual expression mutation. */

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

interface SetExpressionResult {
  readonly value: string;
  readonly type: string;
  readonly variablesReference: number;
  readonly namedVariables?: number;
}

interface DapVariable {
  readonly name: string;
  readonly value: string;
}

/** Verify textual roots and descendants through VS Code's standard DAP request. */
export async function verifyExpressionMutation(
  workspaceRoot: string,
  received: DapMessage[],
  sent: DapMessage[]
): Promise<void> {
  const lines = [
    "program DebuggerExpressionMutation;",
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
  const sourcePath = await writeSource(workspaceRoot, "expression-mutation", lines);
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
      name: "FPAS debugger expression mutation",
      program: sourcePath,
      cwd: workspaceRoot,
      stopOnEntry: false
    });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "stopped"),
      "expression-mutation breakpoint"
    );
    assert.ok(
      sent.slice(marker.sent).some(
        (message) => message.command === "initialize" &&
          message.body?.supportsSetExpression === true
      ),
      "adapter advertises supportsSetExpression"
    );

    await setExpression(session, "Scalar", "10", true);
    const aggregate = await setExpression(
      session,
      "Origin",
      "Origin with X := 20; end",
      true
    );
    assert.equal(aggregate.type, "Point");
    assert.ok(aggregate.variablesReference > 0, "aggregate result is expandable");
    assert.equal(aggregate.namedVariables, 2);
    const fields = await session.customRequest("variables", {
      variablesReference: aggregate.variablesReference,
      start: 0,
      count: 10
    }) as { variables: DapVariable[] };
    assert.ok(fields.variables.some((field) => field.name === "X" && field.value === "20"));

    await setExpression(session, "Items[1]", "30", true);
    await setExpression(session, "Scores['Ada']", "40", true);
    await setExpression(session, "GlobalValue", "50", false);

    await waitFor(
      () => eventCount(sent.slice(marker.sent), "invalidated") >= 5,
      "textual mutation invalidation events"
    );
    const invalidations = eventCount(sent.slice(marker.sent), "invalidated");
    await assert.rejects(
      async () => setExpression(session as vscode.DebugSession, "Fixed", "9", true),
      /not mutable/i
    );
    assert.equal(
      eventCount(sent.slice(marker.sent), "invalidated"),
      invalidations,
      "rejected target emits no invalidation"
    );

    const frameId = await currentFrame(session);
    const fixed = await session.customRequest("evaluate", {
      frameId,
      expression: "Fixed",
      context: "watch"
    }) as { result: string };
    assert.equal(fixed.result, "2", "failed mutation leaves the stop usable");

    await session.customRequest("continue", { threadId: 1 });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "terminated"),
      "expression-mutation termination"
    );
    assert.ok(
      sent.slice(marker.sent).some(
        (message) => message.event === "output" && message.body?.output === "150\n"
      ),
      "continued debuggee observes all textual writes"
    );
    assert.ok(
      received.slice(marker.received).filter(
        (message) => message.command === "setExpression"
      ).length >= 6,
      "Extension Host forwards successful and rejected setExpression requests"
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
  frameScoped: boolean
): Promise<SetExpressionResult> {
  const frameId = frameScoped ? await currentFrame(session) : undefined;
  const result = await session.customRequest("setExpression", {
    expression,
    value,
    frameId
  }) as SetExpressionResult;
  assert.equal(result.value, value === "Origin with X := 20; end" ? "Point {...}" : value);
  return result;
}

async function currentFrame(session: vscode.DebugSession): Promise<number> {
  const stack = await session.customRequest("stackTrace", {
    threadId: 1,
    startFrame: 0,
    levels: 1
  }) as { stackFrames: Array<{ id: number }> };
  const frame = stack.stackFrames[0];
  assert.ok(frame, "stopped session exposes a frame");
  return frame.id;
}
