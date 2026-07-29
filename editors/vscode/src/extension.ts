import * as vscode from "vscode";

/** Command that reveals the Functional Pascal output channel. */
export const SHOW_OUTPUT_COMMAND = "functionalPascal.showOutput";

/** Name of the extension's dedicated output channel. */
export const OUTPUT_CHANNEL_NAME = "Functional Pascal";

/** First line emitted when the extension activates. */
export const ACTIVATION_MESSAGE =
  "Functional Pascal extension activated (Hello World).";

/** Public API exposed to extension-host regression tests. */
export interface FunctionalPascalExtensionApi {
  /** Exact message written to the output channel during activation. */
  readonly activationMessage: string;
}

/** Activates the Functional Pascal extension. */
export function activate(
  context: vscode.ExtensionContext
): FunctionalPascalExtensionApi {
  const outputChannel = vscode.window.createOutputChannel(OUTPUT_CHANNEL_NAME);
  outputChannel.appendLine(ACTIVATION_MESSAGE);

  const showOutput = vscode.commands.registerCommand(SHOW_OUTPUT_COMMAND, () => {
    outputChannel.show(true);
  });

  context.subscriptions.push(outputChannel, showOutput);

  return { activationMessage: ACTIVATION_MESSAGE };
}
