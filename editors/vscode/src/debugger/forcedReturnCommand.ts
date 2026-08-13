/** Interactive forced-return command over the FPAS DAP adapter. */

import * as vscode from "vscode";

/** Stable command identifier contributed by the Functional Pascal extension. */
export const FORCE_RETURN_COMMAND = "functionalPascal.debug.forceReturn";

/** Optional arguments used by command links and Extension Host tests. */
export interface ForcedReturnInput {
  readonly frameId?: number;
  readonly expression?: string;
}

/** Register the editor command that completes the active callee. */
export function registerForcedReturnCommand(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.commands.registerCommand(
      FORCE_RETURN_COMMAND,
      async (input?: ForcedReturnInput) => {
        const selection = await activeSelection(input?.frameId);
        if (selection === undefined) return;
        try {
          if (input?.expression !== undefined) {
            await request(selection, input.expression);
            return;
          }
          try {
            await request(selection, undefined);
          } catch (error) {
            if (!isValueRequired(error)) {
              void vscode.window.showErrorMessage(
                `Functional Pascal forced return failed: ${errorMessage(error)}`
              );
              return;
            }
            const expression = await prompt();
            if (expression === undefined) return;
            await request(selection, expression);
          }
        } catch (error) {
          void vscode.window.showErrorMessage(
            `Functional Pascal forced return failed: ${errorMessage(error)}`
          );
        }
      }
    )
  );
}

interface DebugSelection {
  readonly session: vscode.DebugSession;
  readonly frameId: number;
}

async function activeSelection(frameId?: number): Promise<DebugSelection | undefined> {
  const session = vscode.debug.activeDebugSession;
  if (session?.type !== "fpas") {
    void vscode.window.showWarningMessage(
      "Start and stop a Functional Pascal debug session before forcing a return."
    );
    return undefined;
  }
  if (frameId !== undefined) return { session, frameId };
  const selection = vscode.debug.activeStackItem;
  if (selection instanceof vscode.DebugStackFrame && selection.session === session) {
    return { session, frameId: selection.frameId };
  }
  void vscode.window.showWarningMessage(
    "Select a stopped Functional Pascal stack frame before forcing a return."
  );
  return undefined;
}

async function prompt(): Promise<string | undefined> {
  return vscode.window.showInputBox({
    prompt: "Return expression",
    value: "0",
    ignoreFocusOut: true,
    validateInput: (input) =>
      input.trim().length === 0 ? "Enter one FPAS expression." : undefined
  });
}

async function request(
  selection: DebugSelection,
  expression: string | undefined
): Promise<void> {
  const args: Record<string, unknown> = { frameId: selection.frameId };
  if (expression !== undefined) args.expression = expression;
  const result = await selection.session.customRequest("fpas/forceReturn", args) as {
    value?: string;
  };
  void vscode.window.showInformationMessage(
    `Functional Pascal returned: ${result.value ?? "committed"}`
  );
}

function isValueRequired(error: unknown): boolean {
  return errorMessage(error).includes("requires a return expression");
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
