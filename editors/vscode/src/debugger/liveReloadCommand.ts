/** Interactive live-image reload commands over the FPAS DAP adapter. */

import * as vscode from "vscode";

/** Stable command identifier for rebuilding and installing compatible FPAS code. */
export const LIVE_RELOAD_COMMAND = "functionalPascal.debug.reload";

/** Stable command identifier for restoring the preceding FPAS live image. */
export const LIVE_RELOAD_ROLLBACK_COMMAND =
  "functionalPascal.debug.rollbackReload";

/** Result returned by both live-image commands. */
export interface LiveReloadResult {
  readonly class: string;
  readonly accepted: boolean;
  readonly applied: boolean;
  readonly version: number;
  readonly rollbackAvailable: boolean;
}

/** Register compatible-image reload and one-image rollback commands. */
export function registerLiveReloadCommands(
  context: vscode.ExtensionContext
): void {
  context.subscriptions.push(
    vscode.commands.registerCommand(LIVE_RELOAD_COMMAND, () =>
      runLiveImageRequest("fpas/reload", "reload")
    ),
    vscode.commands.registerCommand(LIVE_RELOAD_ROLLBACK_COMMAND, () =>
      runLiveImageRequest("fpas/reloadRollback", "rollback")
    )
  );
}

async function runLiveImageRequest(
  request: string,
  operation: "reload" | "rollback"
): Promise<LiveReloadResult | undefined> {
  const session = vscode.debug.activeDebugSession;
  if (session?.type !== "fpas") {
    void vscode.window.showWarningMessage(
      "Start a Functional Pascal debug session before reloading its program."
    );
    return undefined;
  }
  try {
    const result = await session.customRequest(request, {}) as LiveReloadResult;
    const action = result.applied ? "applied" : "left unchanged";
    void vscode.window.showInformationMessage(
      `Functional Pascal ${operation} ${action}: ${result.class}, image version ${result.version}.`
    );
    return result;
  } catch (error) {
    void vscode.window.showErrorMessage(
      `Functional Pascal ${operation} failed: ${errorMessage(error)}`
    );
    return undefined;
  }
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
