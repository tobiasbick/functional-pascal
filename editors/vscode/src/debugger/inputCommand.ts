/** Interactive queued debuggee input for hosted Read/ReadLn. */

import * as vscode from "vscode";

/** Stable command identifier for sending one program input line. */
export const SEND_INPUT_COMMAND = "functionalPascal.debug.sendInput";

/** Stable command identifier for signaling program input EOF. */
export const SIGNAL_INPUT_EOF_COMMAND = "functionalPascal.debug.signalInputEof";

/** Stable command identifier for dropping unread queued program input. */
export const CANCEL_INPUT_COMMAND = "functionalPascal.debug.cancelInput";

/** Optional arguments used by command links and Extension Host tests. */
export interface DebuggeeInput {
  readonly text?: string;
}

/** Register editor commands that map to `fpas/input`, `fpas/eof`, and `fpas/cancelInput`. */
export function registerDebuggeeInputCommands(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.commands.registerCommand(
      SEND_INPUT_COMMAND,
      async (input?: DebuggeeInput) => {
        const session = activeSession();
        if (session === undefined) return;
        const text = input?.text ?? await prompt();
        if (text === undefined) return;
        try {
          await session.customRequest("fpas/input", { text });
        } catch (error) {
          void vscode.window.showErrorMessage(
            `Functional Pascal program input failed: ${errorMessage(error)}`
          );
        }
      }
    ),
    vscode.commands.registerCommand(
      SIGNAL_INPUT_EOF_COMMAND,
      async () => {
        const session = activeSession();
        if (session === undefined) return;
        try {
          await session.customRequest("fpas/eof", {});
        } catch (error) {
          void vscode.window.showErrorMessage(
            `Functional Pascal input EOF failed: ${errorMessage(error)}`
          );
        }
      }
    ),
    vscode.commands.registerCommand(
      CANCEL_INPUT_COMMAND,
      async () => {
        const session = activeSession();
        if (session === undefined) return;
        try {
          await session.customRequest("fpas/cancelInput", {});
        } catch (error) {
          void vscode.window.showErrorMessage(
            `Functional Pascal input cancel failed: ${errorMessage(error)}`
          );
        }
      }
    )
  );
}

function activeSession(): vscode.DebugSession | undefined {
  const session = vscode.debug.activeDebugSession;
  if (session?.type !== "fpas") {
    void vscode.window.showWarningMessage(
      "Start and stop a Functional Pascal debug session before sending program input."
    );
    return undefined;
  }
  return session;
}

async function prompt(): Promise<string | undefined> {
  return vscode.window.showInputBox({
    prompt: "Line for Std.Console.ReadLn",
    ignoreFocusOut: true
  });
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
