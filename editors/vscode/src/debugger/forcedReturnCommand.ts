/** Interactive forced-return command over the FPAS DAP adapter. */

import * as vscode from "vscode";
import { activeSelection, type DebugSelection } from "./commands/selection";
import { promptExpression as prompt } from "./commands/expressionPrompt";

/** Stable command identifier contributed by the Functional Pascal extension. */
export const FORCE_RETURN_COMMAND = "functionalPascal.debug.forceReturn";

/** Optional arguments used by command links and Extension Host tests. */
export interface ForcedReturnInput {
  readonly frameId?: number;
  readonly expression?: string;
}

/** Register the editor command that completes the selected frame or task entry. */
export function registerForcedReturnCommand(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.commands.registerCommand(
      FORCE_RETURN_COMMAND,
      async (input?: ForcedReturnInput) => {
        const selection = await activeSelection(input?.frameId, "forcing a return");
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
            const expression = await prompt("Return expression", "0");
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


async function request(
  selection: DebugSelection,
  expression: string | undefined
): Promise<void> {
  const args: Record<string, unknown> = { frameId: selection.frameId };
  if (expression !== undefined) args.expression = expression;
  const result = await selection.session.customRequest("fpas/forceReturn", args) as {
    value?: string;
    unwoundFrames?: number;
  };
  const frames = result.unwoundFrames === undefined
    ? ""
    : ` (${result.unwoundFrames} ${result.unwoundFrames === 1 ? "frame" : "frames"})`;
  void vscode.window.showInformationMessage(
    `Functional Pascal returned: ${result.value ?? "committed"}${frames}`
  );
}

function isValueRequired(error: unknown): boolean {
  return errorMessage(error).includes("requires a return expression");
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
