/** Interactive replacement of one retained completed FPAS task result. */

import * as vscode from "vscode";

/** Stable command identifier contributed by the Functional Pascal extension. */
export const REPLACE_TASK_RESULT_COMMAND = "functionalPascal.debug.replaceTaskResult";

/** Optional arguments used by command links and Extension Host tests. */
export interface TaskResultInput {
  readonly taskId?: number;
  readonly frameId?: number;
  readonly expression?: string;
}

/** Register the editor command for replacing an unconsumed completed task result. */
export function registerTaskResultCommand(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.commands.registerCommand(
      REPLACE_TASK_RESULT_COMMAND,
      async (input?: TaskResultInput) => {
        const session = vscode.debug.activeDebugSession;
        if (session?.type !== "fpas") {
          void vscode.window.showWarningMessage(
            "Start and stop a Functional Pascal debug session before replacing a task result."
          );
          return;
        }
        const taskId = input?.taskId ?? await promptTaskId();
        if (taskId === undefined) return;
        const frameId = input?.frameId ?? activeFrameId(session);
        try {
          if (input?.expression !== undefined) {
            await request(session, taskId, frameId, input.expression);
            return;
          }
          try {
            await request(session, taskId, frameId, undefined);
          } catch (error) {
            if (!isValueRequired(error)) throw error;
            const expression = await promptExpression();
            if (expression === undefined) return;
            await request(session, taskId, frameId, expression);
          }
        } catch (error) {
          void vscode.window.showErrorMessage(
            `Functional Pascal task-result replacement failed: ${errorMessage(error)}`
          );
        }
      }
    )
  );
}

function activeFrameId(session: vscode.DebugSession): number | undefined {
  const selection = vscode.debug.activeStackItem;
  return selection instanceof vscode.DebugStackFrame && selection.session === session
    ? selection.frameId
    : undefined;
}

async function promptTaskId(): Promise<number | undefined> {
  const source = await vscode.window.showInputBox({
    prompt: "Completed retained task ID",
    value: "1",
    ignoreFocusOut: true,
    validateInput: (input) => parseTaskId(input) === undefined
      ? "Enter a non-negative integer task ID."
      : undefined
  });
  return source === undefined ? undefined : parseTaskId(source);
}

function parseTaskId(source: string): number | undefined {
  const trimmed = source.trim();
  if (!/^\d+$/.test(trimmed)) return undefined;
  const value = Number(trimmed);
  return Number.isSafeInteger(value) ? value : undefined;
}

async function promptExpression(): Promise<string | undefined> {
  return vscode.window.showInputBox({
    prompt: "Replacement result expression",
    value: "0",
    ignoreFocusOut: true,
    validateInput: (input) => input.trim().length === 0
      ? "Enter one FPAS expression."
      : undefined
  });
}

async function request(
  session: vscode.DebugSession,
  taskId: number,
  frameId: number | undefined,
  expression: string | undefined
): Promise<void> {
  const arguments_: Record<string, unknown> = { taskId };
  if (frameId !== undefined) arguments_.frameId = frameId;
  if (expression !== undefined) arguments_.expression = expression;
  const result = await session.customRequest("fpas/replaceTaskResult", arguments_) as {
    value?: string;
  };
  void vscode.window.showInformationMessage(
    `Functional Pascal task ${taskId} result replaced: ${result.value ?? "committed"}`
  );
}

function isValueRequired(error: unknown): boolean {
  return errorMessage(error).includes("requires a replacement expression");
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
