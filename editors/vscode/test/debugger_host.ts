/** Real Extension Host debugger lifecycle verification. */

import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";

import * as vscode from "vscode";

interface DapMessage {
  readonly type?: string;
  readonly command?: string;
  readonly event?: string;
  readonly success?: boolean;
  readonly body?: Record<string, unknown>;
}

/** Launch, step, inspect, observe output, and terminate through VS Code's debug API. */
export async function verifyDebuggerHost(): Promise<void> {
  const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  assert.ok(workspaceRoot, "extension test workspace is open");
  const sourcePath = path.join(
    workspaceRoot,
    `.debugger-host-${process.pid}-${Date.now()}.fpas`
  );
  const source = [
    "program DebuggerHost;",
    "",
    "uses Std.Console;",
    "",
    "begin",
    "  mutable var Value: integer := 1;",
    "  Value := Value + 1;",
    "  WriteLn(Value)",
    "end.",
    ""
  ].join("\n");
  await fs.writeFile(sourcePath, source);

  const received: DapMessage[] = [];
  const sent: DapMessage[] = [];
  const tracker = vscode.debug.registerDebugAdapterTrackerFactory("fpas", {
    createDebugAdapterTracker: () => ({
      onWillReceiveMessage: (message: DapMessage) => received.push(message),
      onDidSendMessage: (message: DapMessage) => sent.push(message)
    })
  });
  const sourceUri = vscode.Uri.file(sourcePath);
  const breakpoint = new vscode.SourceBreakpoint(
    new vscode.Location(sourceUri, new vscode.Position(6, 2))
  );
  vscode.debug.addBreakpoints([breakpoint]);

  try {
    await vscode.window.showTextDocument(await vscode.workspace.openTextDocument(sourceUri));
    await vscode.commands.executeCommand("workbench.view.debug");
    const started = await vscode.debug.startDebugging(undefined, {
      type: "fpas",
      request: "launch",
      name: "FPAS debugger host test",
      program: sourcePath,
      cwd: workspaceRoot,
      stopOnEntry: true
    });
    assert.equal(started, true);
    await waitFor(() => eventCount(sent, "stopped") >= 1, "entry stop");
    await vscode.commands.executeCommand("workbench.debug.action.focusVariablesView");
    await waitFor(
      () => received.some((message) => message.command === "variables"),
      "variable inspection"
    );

    await vscode.commands.executeCommand("workbench.action.debug.stepOver");
    await waitFor(() => eventCount(sent, "stopped") >= 2, "step stop");
    await vscode.commands.executeCommand("workbench.action.debug.continue");
    await waitFor(
      () => sent.some((message) => message.event === "terminated"),
      "debug termination"
    );

    for (const command of ["initialize", "launch", "setBreakpoints", "configurationDone", "threads", "stackTrace", "scopes", "variables", "next", "continue"]) {
      assert.ok(received.some((message) => message.command === command), `DAP request ${command}`);
    }
    assert.ok(sent.some((message) => message.event === "output"));
    assert.ok(sent.some((message) => message.event === "stopped" && message.body?.reason === "breakpoint"));
    assert.ok(sent.filter((message) => message.type === "response").every((message) => message.success !== false));
  } finally {
    vscode.debug.removeBreakpoints([breakpoint]);
    await vscode.debug.stopDebugging();
    tracker.dispose();
    await vscode.commands.executeCommand("workbench.action.closeActiveEditor");
    await fs.rm(sourcePath, { force: true });
  }
}

function eventCount(messages: readonly DapMessage[], event: string): number {
  return messages.filter((message) => message.event === event).length;
}

async function waitFor(predicate: () => boolean, label: string): Promise<void> {
  const deadline = Date.now() + 15_000;
  while (Date.now() < deadline) {
    if (predicate()) return;
    await new Promise((resolve) => setTimeout(resolve, 25));
  }
  assert.fail(`timed out waiting for ${label}`);
}
