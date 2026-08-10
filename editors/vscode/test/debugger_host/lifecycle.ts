/** Breakpoint, stepping, inspection, source, and output host coverage. */

import assert from "node:assert/strict";

import * as vscode from "vscode";

import {
  closeAndRemoveSource,
  type DapMessage,
  startSession,
  waitFor,
  waitForStoppedReady,
  writeSource
} from "./support";

interface DapStackFrame {
  readonly id: number;
  readonly name: string;
}

interface DapScope {
  readonly name: string;
  readonly variablesReference: number;
}

interface DapVariable {
  readonly name: string;
  readonly value: string;
  readonly variablesReference: number;
}

export async function verifyDebuggerLifecycle(
  workspaceRoot: string,
  received: DapMessage[],
  sent: DapMessage[]
): Promise<void> {
  const sourceLines = [
    "program DebuggerHost;",
    "",
    "uses Std.Console;",
    "",
    "type",
    "  Point = record",
    "    X: integer;",
    "    Y: integer;",
    "  end;",
    "",
    "function Factorial(Value: integer): integer;",
    "begin",
    "  if Value <= 1 then",
    "    return 1;",
    "  return Value * Factorial(Value - 1)",
    "end;",
    "",
    "begin",
    "  var Origin: Point := record",
    "    X := 3;",
    "    Y := 4;",
    "  end;",
    "  mutable var Computed: integer := Factorial(4);",
    "  Computed := Computed + Origin.X;",
    "  WriteLn(Computed)",
    "end.",
    ""
  ];
  const sourcePath = await writeSource(workspaceRoot, "lifecycle", sourceLines);
  const sourceUri = vscode.Uri.file(sourcePath);
  const marker = { received: received.length, sent: sent.length };
  let breakpoint: vscode.SourceBreakpoint | undefined;
  let session: vscode.DebugSession | undefined;

  try {
    const editor = await vscode.window.showTextDocument(
      await vscode.workspace.openTextDocument(sourceUri)
    );
    const breakpointLine = sourceLines.indexOf(
      "  return Value * Factorial(Value - 1)"
    );
    editor.selection = new vscode.Selection(
      breakpointLine,
      2,
      breakpointLine,
      2
    );
    await vscode.commands.executeCommand("editor.debug.action.toggleBreakpoint");
    await waitFor(
      () =>
        vscode.debug.breakpoints.some((candidate) =>
          isSourceBreakpoint(candidate, sourcePath, breakpointLine)
        ),
      "F9 source breakpoint registration"
    );
    breakpoint = vscode.debug.breakpoints.find((candidate) =>
      isSourceBreakpoint(candidate, sourcePath, breakpointLine)
    ) as vscode.SourceBreakpoint;

    session = await startSession({
      type: "fpas",
      request: "launch",
      name: "FPAS debugger V1 lifecycle",
      program: sourcePath,
      cwd: workspaceRoot,
      stopOnEntry: true
    });
    await waitForStoppedReady(() => sent.slice(marker.sent), 1, "entry stop");

    await vscode.commands.executeCommand("workbench.action.debug.stepInto");
    await waitForStoppedReady(() => sent.slice(marker.sent), 2, "step-in stop");

    await vscode.commands.executeCommand("workbench.action.debug.stepOver");
    await waitForStoppedReady(() => sent.slice(marker.sent), 3, "step-over stop");

    await vscode.commands.executeCommand("workbench.action.debug.continue");
    await waitForStoppedReady(
      () => sent.slice(marker.sent),
      4,
      "source breakpoint stop"
    );
    assert.ok(
      sent.slice(marker.sent).some(
        (message) =>
          message.event === "stopped" && message.body?.reason === "breakpoint"
      ),
      "source breakpoint reports its stop reason"
    );
    const stackAtBreakpoint = await session.customRequest("stackTrace", {
      threadId: 1,
      startFrame: 0,
      levels: 64
    }) as { stackFrames: DapStackFrame[] };
    assert.ok(
      stackAtBreakpoint.stackFrames.length >= 2,
      "breakpoint inside Factorial exposes caller and callee frames"
    );
    await verifyInspection(session, sourcePath, sourceLines.join("\n"));

    vscode.debug.removeBreakpoints([breakpoint]);
    breakpoint = undefined;
    await vscode.commands.executeCommand("workbench.action.debug.stepOut");
    await waitForStoppedReady(
      () => sent.slice(marker.sent),
      5,
      "step-out stop"
    );

    await vscode.commands.executeCommand("workbench.action.debug.continue");
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "terminated"),
      "debug termination"
    );
    verifyLifecycleTranscript(
      received.slice(marker.received),
      sent.slice(marker.sent)
    );
  } finally {
    if (breakpoint) vscode.debug.removeBreakpoints([breakpoint]);
    if (session) await vscode.debug.stopDebugging(session);
    await closeAndRemoveSource(sourcePath);
  }
}

async function verifyInspection(
  session: vscode.DebugSession,
  sourcePath: string,
  expectedSource: string
): Promise<void> {
  const threads = await session.customRequest("threads") as {
    threads: Array<{ id: number }>;
  };
  assert.deepEqual(threads.threads.map((thread) => thread.id), [1]);

  const stack = await session.customRequest("stackTrace", {
    threadId: 1,
    startFrame: 0,
    levels: 64
  }) as { stackFrames: DapStackFrame[] };
  const rootFrame = stack.stackFrames.at(-1);
  assert.ok(rootFrame, "stopped session exposes its root frame");

  const scopes = await session.customRequest("scopes", {
    frameId: rootFrame.id
  }) as { scopes: DapScope[] };
  const locals = scopes.scopes.find((scope) => scope.name === "Locals");
  assert.ok(locals, "stopped frame exposes locals");
  const variables = await session.customRequest("variables", {
    variablesReference: locals.variablesReference,
    start: 0,
    count: 100
  }) as { variables: DapVariable[] };
  const origin = variables.variables.find((variable) => variable.name === "Origin");
  assert.ok(origin, "initialized record local is visible");
  assert.ok(origin.variablesReference > 0, "record local is expandable");
  const fields = await session.customRequest("variables", {
    variablesReference: origin.variablesReference,
    start: 0,
    count: 100
  }) as { variables: DapVariable[] };
  assert.deepEqual(
    fields.variables.map((field) => [field.name, field.value]),
    [["X", "3"], ["Y", "4"]]
  );

  const source = await session.customRequest("source", {
    source: { path: sourcePath },
    sourceReference: 0
  }) as { content: string };
  assert.equal(source.content, expectedSource);
}

function verifyLifecycleTranscript(
  received: readonly DapMessage[],
  sent: readonly DapMessage[]
): void {
  for (const command of [
    "initialize",
    "launch",
    "setBreakpoints",
    "configurationDone",
    "threads",
    "stackTrace",
    "scopes",
    "variables",
    "source",
    "stepIn",
    "next",
    "stepOut",
    "continue"
  ]) {
    assert.ok(
      received.some((message) => message.command === command),
      `DAP request ${command}`
    );
  }
  assert.ok(
    sent.some((message) => message.event === "output"),
    "program output reaches VS Code"
  );
  const failed = sent.filter(
    (message) => message.type === "response" && message.success === false
  );
  assert.deepEqual(failed, [], "supported lifecycle requests succeed");
}

function isSourceBreakpoint(
  candidate: vscode.Breakpoint,
  sourcePath: string,
  line: number
): candidate is vscode.SourceBreakpoint {
  return (
    candidate instanceof vscode.SourceBreakpoint &&
    candidate.location.uri.fsPath === sourcePath &&
    candidate.location.range.start.line === line
  );
}
