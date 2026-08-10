/** Shared orchestration helpers for real VS Code debugger sessions. */

import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";

import * as vscode from "vscode";

export interface DapMessage {
  readonly type?: string;
  readonly command?: string;
  readonly event?: string;
  readonly success?: boolean;
  readonly arguments?: Record<string, unknown>;
  readonly body?: Record<string, unknown>;
}

export async function startSession(
  configuration: vscode.DebugConfiguration
): Promise<vscode.DebugSession> {
  const started = await vscode.debug.startDebugging(undefined, configuration);
  assert.equal(started, true, `debug session ${configuration.name} starts`);
  await waitFor(
    () => vscode.debug.activeDebugSession?.name === configuration.name,
    `active session ${configuration.name}`
  );
  const session = vscode.debug.activeDebugSession;
  assert.ok(session, `debug session ${configuration.name} is active`);
  return session;
}

export async function writeSource(
  workspaceRoot: string,
  scenario: string,
  lines: string[]
): Promise<string> {
  const sourcePath = path.join(
    workspaceRoot,
    `.debugger-host-${scenario}-${process.pid}-${Date.now()}.fpas`
  );
  await fs.writeFile(sourcePath, lines.join("\n"));
  return sourcePath;
}

export async function closeAndRemoveSource(sourcePath: string): Promise<void> {
  const activePath = vscode.window.activeTextEditor?.document.uri.fsPath;
  if (activePath === sourcePath) {
    await vscode.commands.executeCommand("workbench.action.closeActiveEditor");
  }
  await fs.rm(sourcePath, { force: true });
}

export function eventCount(
  messages: readonly DapMessage[],
  event: string
): number {
  return messages.filter((message) => message.event === event).length;
}

function responseCount(
  messages: readonly DapMessage[],
  command: string
): number {
  return messages.filter(
    (message) => message.type === "response" && message.command === command
  ).length;
}

export async function waitForStoppedReady(
  messages: () => readonly DapMessage[],
  expectedStops: number,
  label: string
): Promise<void> {
  await waitFor(
    () =>
      eventCount(messages(), "stopped") >= expectedStops &&
      responseCount(messages(), "stackTrace") >= expectedStops,
    label
  );
}

export async function waitFor(
  predicate: () => boolean,
  label: string
): Promise<void> {
  const deadline = Date.now() + 15_000;
  while (Date.now() < deadline) {
    if (predicate()) return;
    await new Promise((resolve) => setTimeout(resolve, 25));
  }
  assert.fail(`timed out waiting for ${label}`);
}
