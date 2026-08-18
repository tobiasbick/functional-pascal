/** Real Extension Host coverage for compatible FPAS live-image reload. */

import assert from "node:assert/strict";
import fs from "node:fs/promises";

import * as vscode from "vscode";

import {
  LIVE_RELOAD_COMMAND,
  LIVE_RELOAD_ROLLBACK_COMMAND,
  type LiveReloadResult
} from "../../src/debugger/liveReloadCommand";
import {
  closeAndRemoveSource,
  type DapMessage,
  startSession,
  waitFor,
  waitForStoppedReady,
  writeSource
} from "./support";

/** Verify rebuild, compatible commit, rollback, and execution of reloaded code. */
export async function verifyLiveReload(
  workspaceRoot: string,
  received: DapMessage[],
  sent: DapMessage[]
): Promise<void> {
  const initial = sourceWithValue(1);
  const sourcePath = await writeSource(workspaceRoot, "live-reload", initial);
  const marker = { received: received.length, sent: sent.length };
  let session: vscode.DebugSession | undefined;
  try {
    session = await startSession({
      type: "fpas",
      request: "launch",
      name: "FPAS live reload",
      program: sourcePath,
      cwd: workspaceRoot,
      stopOnEntry: true
    });
    await waitForStoppedReady(() => sent.slice(marker.sent), 1, "live-reload entry stop");
    const initialize = sent
      .slice(marker.sent)
      .find((message) => message.type === "response" && message.command === "initialize");
    assert.equal(initialize?.body?.supportsHotReload, true);

    await fs.writeFile(sourcePath, sourceWithValue(2).join("\n"));
    const committed = await vscode.commands.executeCommand<LiveReloadResult>(
      LIVE_RELOAD_COMMAND
    );
    assert.deepEqual(reloadIdentity(committed), {
      class: "inactive_function_body",
      applied: true,
      version: 2,
      rollbackAvailable: true
    });

    const rolledBack = await vscode.commands.executeCommand<LiveReloadResult>(
      LIVE_RELOAD_ROLLBACK_COMMAND
    );
    assert.deepEqual(reloadIdentity(rolledBack), {
      class: "inactive_function_body",
      applied: true,
      version: 3,
      rollbackAvailable: true
    });

    const recommitted = await vscode.commands.executeCommand<LiveReloadResult>(
      LIVE_RELOAD_COMMAND
    );
    assert.deepEqual(reloadIdentity(recommitted), {
      class: "inactive_function_body",
      applied: true,
      version: 4,
      rollbackAvailable: true
    });
    assert.ok(
      received
        .slice(marker.received)
        .some((message) => message.command === "fpas/reloadRollback")
    );
    assert.ok(
      sent
        .slice(marker.sent)
        .filter((message) => message.event === "invalidated").length >= 3
    );

    await session.customRequest("continue", { threadId: 1 });
    await waitFor(
      () => sent.slice(marker.sent).some((message) => message.event === "terminated"),
      "live-reload termination"
    );
    assert.equal(output(sent.slice(marker.sent)), "2\n");
  } finally {
    if (session) await vscode.debug.stopDebugging(session);
    await closeAndRemoveSource(sourcePath);
  }
}

function sourceWithValue(value: number): string[] {
  return [
    "program DebuggerLiveReload;",
    "",
    "uses Std.Console;",
    "",
    "function Value(): integer;",
    "begin",
    `  return ${value}`,
    "end;",
    "",
    "begin",
    "  WriteLn(Value())",
    "end.",
    ""
  ];
}

function reloadIdentity(result: LiveReloadResult | undefined): object {
  assert.ok(result);
  return {
    class: result.class,
    applied: result.applied,
    version: result.version,
    rollbackAvailable: result.rollbackAvailable
  };
}

function output(messages: readonly DapMessage[]): string {
  return messages
    .filter((message) => message.event === "output")
    .map((message) => String(message.body?.output ?? ""))
    .join("");
}
