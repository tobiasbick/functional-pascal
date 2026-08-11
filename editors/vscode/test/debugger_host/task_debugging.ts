/** Multi-task Threads, inspection, mutation, stepping, and lifecycle host coverage. */

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

interface DapThread {
  readonly id: number;
  readonly name: string;
}

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
}

/** Verify that VS Code can operate on distinct FPAS task identities. */
export async function verifyTaskDebugging(
  workspaceRoot: string,
  received: DapMessage[],
  sent: DapMessage[]
): Promise<void> {
  const sourceLines = [
    "program TaskDebuggerHost;",
    "",
    "uses Std.Console, Std.Task;",
    "",
    "function Work(Start: integer): integer;",
    "begin",
    "  mutable var Value: integer := Start;",
    "  Value := Value + 1;",
    "  return Value",
    "end;",
    "",
    "begin",
    "  var First: task := go Work(10);",
    "  var Second: task := go Work(20);",
    "  var Pending: array of task := [First, Second];",
    "  WaitAll(Pending);",
    "  WriteLn(Wait(First));",
    "  WriteLn(Wait(Second))",
    "end.",
    ""
  ];
  const sourcePath = await writeSource(workspaceRoot, "tasks", sourceLines);
  const marker = { received: received.length, sent: sent.length };
  const breakpointLine = sourceLines.indexOf("  Value := Value + 1;");
  const breakpoint = new vscode.SourceBreakpoint(
    new vscode.Location(vscode.Uri.file(sourcePath), new vscode.Position(breakpointLine, 2))
  );
  let session: vscode.DebugSession | undefined;

  vscode.debug.addBreakpoints([breakpoint]);
  try {
    session = await startSession({
      type: "fpas",
      request: "launch",
      name: "FPAS task debugging",
      program: sourcePath,
      cwd: workspaceRoot,
      stopOnEntry: false
    });
    await waitForStoppedReady(() => sent.slice(marker.sent), 1, "first task breakpoint");

    const firstStop = stoppedEvents(sent.slice(marker.sent))[0];
    const firstThread = numberBodyField(firstStop, "threadId");
    assert.notEqual(firstThread, 1, "spawned task has a distinct DAP thread ID");
    assert.equal(firstStop.body?.allThreadsStopped, true);

    const threads = await session.customRequest("threads") as { threads: DapThread[] };
    assert.ok(threads.threads.some((thread) => thread.id === 1 && thread.name === "FPAS main"));
    assert.ok(threads.threads.some((thread) => thread.id === firstThread));

    const mainStack = await session.customRequest("stackTrace", {
      threadId: 1,
      startFrame: 0,
      levels: 64
    }) as { stackFrames: DapStackFrame[] };
    assert.equal(mainStack.stackFrames[0]?.name, "taskdebuggerhost");

    const taskStack = await session.customRequest("stackTrace", {
      threadId: firstThread,
      startFrame: 0,
      levels: 64
    }) as { stackFrames: DapStackFrame[] };
    const taskFrame = taskStack.stackFrames[0];
    assert.equal(taskFrame?.name, "work");

    const scopes = await session.customRequest("scopes", {
      frameId: taskFrame.id
    }) as { scopes: DapScope[] };
    const locals = scopes.scopes.find((scope) => scope.name === "Locals");
    assert.ok(locals, "selected task exposes local variables");
    const variables = await session.customRequest("variables", {
      variablesReference: locals.variablesReference,
      start: 0,
      count: 100
    }) as { variables: DapVariable[] };
    assert.ok(
      variables.variables.some(
        (variable) => variable.name === "Value" && variable.value === "10"
      ),
      "selected task local has its own value"
    );
    const evaluated = await session.customRequest("evaluate", {
      frameId: taskFrame.id,
      expression: "Value",
      context: "watch"
    }) as { result: string };
    assert.equal(evaluated.result, "10");
    const mutated = await session.customRequest("setVariable", {
      variablesReference: locals.variablesReference,
      name: "Value",
      value: "15"
    }) as { value: string };
    assert.equal(mutated.value, "15");

    await session.customRequest("stepIn", { threadId: firstThread });
    await waitForStoppedReady(() => sent.slice(marker.sent), 2, "selected task step");
    const stepStop = stoppedEvents(sent.slice(marker.sent))[1];
    assert.equal(stepStop.body?.reason, "step");
    assert.equal(numberBodyField(stepStop, "threadId"), firstThread);

    await session.customRequest("continue", { threadId: firstThread });
    await waitForStoppedReady(() => sent.slice(marker.sent), 3, "second task breakpoint");
    const secondStop = stoppedEvents(sent.slice(marker.sent))[2];
    const secondThread = numberBodyField(secondStop, "threadId");
    assert.equal(secondStop.body?.reason, "breakpoint");
    assert.notEqual(secondThread, firstThread, "another task owns the next breakpoint");

    vscode.debug.removeBreakpoints([breakpoint]);
    await session.customRequest("continue", { threadId: secondThread });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "terminated"),
      "task debug termination"
    );
    const transcript = sent.slice(marker.sent);
    const started = transcript.filter(
      (message) => message.event === "thread" && message.body?.reason === "started"
    );
    const exited = transcript.filter(
      (message) => message.event === "thread" && message.body?.reason === "exited"
    );
    assert.equal(started.length, 2, "each spawned task starts once");
    assert.equal(exited.length, 2, "each spawned task exits once");
    assert.ok(
      transcript.some(
        (message) => message.event === "output" && message.body?.output === "16\n"
      ),
      "mutation affects only the selected task before it resumes"
    );
  } finally {
    vscode.debug.removeBreakpoints([breakpoint]);
    if (session) await vscode.debug.stopDebugging(session);
    await closeAndRemoveSource(sourcePath);
  }
}

function stoppedEvents(messages: readonly DapMessage[]): DapMessage[] {
  return messages.filter((message) => message.event === "stopped");
}

function numberBodyField(message: DapMessage, name: string): number {
  const value = message.body?.[name];
  if (typeof value !== "number") {
    assert.fail(`${name} is numeric`);
  }
  return value;
}
