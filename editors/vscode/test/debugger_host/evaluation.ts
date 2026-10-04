/** Watch, hover, variables-view, and Debug Console evaluation coverage. */

import assert from "node:assert/strict";

import * as vscode from "vscode";

import {
  closeAndRemoveSource,
  type DapMessage,
  startSession,
  waitFor,
  writeSource
} from "./support";

interface EvaluateResult {
  readonly result: string;
  readonly type: string;
  readonly variablesReference: number;
}

export async function verifyDebuggerEvaluation(
  workspaceRoot: string,
  received: DapMessage[],
  sent: DapMessage[]
): Promise<void> {
  const lines = [
    "program DebuggerEvaluation;",
    "",
    " uses Std.Console as Console;",
    "",
    "",
    "  type Point = record",
    "    X: integer;",
    "    Y: integer;",
    "  end record;",
    "    function PointCreate(X: integer; Y: integer): Point;",
    "    begin",
    "      return Point(X := X, Y := Y);",
    "    end function;",
    "    function PointSum(Receiver: Point): integer;",
    "    begin",
    "      return Receiver.X + Receiver.Y;",
    "    end function;",
    "    function PointReadFirst(Receiver: Point): integer;",
    "    begin",
    "      return Receiver.X;",
    "    end function;",
    "",
    "begin",
    "  const Origin: Point := Point(",
    "    X := 3,",
    "    Y := 4",
    "  );",
    "  const Offset: integer := 2;",
    "  Console.WriteLn(Offset);",
    "end program;",
    ""
  ];
  const sourcePath = await writeSource(workspaceRoot, "evaluation", lines);
  const line = lines.indexOf("  Console.WriteLn(Offset);");
  const breakpoint = new vscode.SourceBreakpoint(
    new vscode.Location(vscode.Uri.file(sourcePath), new vscode.Position(line, 2))
  );
  const marker = { received: received.length, sent: sent.length };
  vscode.debug.addBreakpoints([breakpoint]);
  let session: vscode.DebugSession | undefined;
  try {
    session = await startSession({
      type: "fpas",
      request: "launch",
      name: "FPAS debugger evaluation",
      program: sourcePath,
      cwd: workspaceRoot,
      stopOnEntry: false
    });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "stopped"),
      "evaluation breakpoint"
    );
    const stack = await session.customRequest("stackTrace", { threadId: 1 }) as {
      stackFrames: Array<{ id: number }>;
    };
    const frameId = stack.stackFrames[0]?.id;
    assert.ok(frameId, "evaluation stop exposes a frame");
    for (const context of ["watch", "repl", "hover", "variables"]) {
      const result = await session.customRequest("evaluate", {
        expression: "PointSum(PointCreate(Offset, Origin.X))",
        frameId,
        context
      }) as EvaluateResult;
      assert.equal(result.result, "5", `${context} uses the shared evaluator`);
      assert.equal(result.type, "integer");
    }
    const aggregate = await session.customRequest("evaluate", {
      expression: "Origin",
      frameId,
      context: "watch"
    }) as EvaluateResult;
    assert.ok(aggregate.variablesReference > 0, "evaluated record is expandable");
    const children = await session.customRequest("variables", {
      variablesReference: aggregate.variablesReference
    }) as { variables: Array<{ name: string; value: string }> };
    assert.deepEqual(children.variables.map((value) => [value.name, value.value]), [
      ["X", "3"],
      ["Y", "4"]
    ]);
    const accessor = await session.customRequest("evaluate", {
      expression: "PointReadFirst(PointCreate(6, 7))",
      frameId,
      context: "hover"
    }) as EvaluateResult;
    assert.equal(accessor.result, "6", "ordinary accessor calls preserve record arguments");
    const activeSession = session;
    await assert.rejects(
      async () => activeSession.customRequest("evaluate", {
        expression: "Std.Console.WriteLn(Offset)",
        frameId,
        context: "repl"
      }),
      /forbidden|effect|host/i
    );
    const initialize = sent.slice(marker.sent).find(
      (message) => message.type === "response" && message.command === "initialize"
    );
    assert.equal(initialize?.body?.supportsEvaluateForHovers, true);
    await session.customRequest("continue", { threadId: 1 });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "terminated"),
      "evaluation session termination"
    );
    assert.ok(
      received.slice(marker.received).filter((message) => message.command === "evaluate").length >= 6,
      "real Extension Host forwards every evaluation context"
    );
  } finally {
    vscode.debug.removeBreakpoints([breakpoint]);
    if (session) await vscode.debug.stopDebugging(session);
    await closeAndRemoveSource(sourcePath);
  }
}
