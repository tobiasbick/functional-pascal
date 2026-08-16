/** Interactive per-task pause, resume, and cancel for the selected debug thread. */

import * as vscode from "vscode";

/** Stable command identifier for holding one FPAS task. */
export const PAUSE_TASK_COMMAND = "functionalPascal.debug.pauseTask";

/** Stable command identifier for releasing one FPAS task hold. */
export const RESUME_TASK_COMMAND = "functionalPascal.debug.resumeTask";

/** Stable command identifier for cancelling one live non-root FPAS task. */
export const CANCEL_TASK_COMMAND = "functionalPascal.debug.cancelTask";

/** Optional arguments used by command links and Extension Host tests. */
export interface TaskControlInput {
  readonly threadId?: number;
}

/** Register editor commands that map to `fpas/pauseTask`, `fpas/resumeTask`, and `fpas/cancelTask`. */
export function registerTaskControlCommands(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.commands.registerCommand(
      PAUSE_TASK_COMMAND,
      async (input?: TaskControlInput) => {
        await runTaskRequest("fpas/pauseTask", "pause", input);
      }
    ),
    vscode.commands.registerCommand(
      RESUME_TASK_COMMAND,
      async (input?: TaskControlInput) => {
        await runTaskRequest("fpas/resumeTask", "resume", input);
      }
    ),
    vscode.commands.registerCommand(
      CANCEL_TASK_COMMAND,
      async (input?: TaskControlInput) => {
        await runTaskRequest("fpas/cancelTask", "cancel", input);
      }
    )
  );
}

async function runTaskRequest(
  request: "fpas/pauseTask" | "fpas/resumeTask" | "fpas/cancelTask",
  action: "pause" | "resume" | "cancel",
  input?: TaskControlInput
): Promise<void> {
  const session = vscode.debug.activeDebugSession;
  if (session?.type !== "fpas") {
    void vscode.window.showWarningMessage(
      "Start and stop a Functional Pascal debug session before controlling a task."
    );
    return;
  }
  const threadId = input?.threadId ?? activeThreadId(session);
  if (threadId === undefined) {
    void vscode.window.showWarningMessage(
      "Select a Functional Pascal task in the Threads view first."
    );
    return;
  }
  try {
    await session.customRequest(request, { threadId });
  } catch (error) {
    void vscode.window.showErrorMessage(
      `Functional Pascal task ${action} failed: ${errorMessage(error)}`
    );
  }
}

function activeThreadId(session: vscode.DebugSession): number | undefined {
  const selection = vscode.debug.activeStackItem;
  if (selection instanceof vscode.DebugThread && selection.session === session) {
    return selection.threadId;
  }
  if (selection instanceof vscode.DebugStackFrame && selection.session === session) {
    return selection.threadId;
  }
  return undefined;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
