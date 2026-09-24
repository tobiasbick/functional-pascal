import * as vscode from "vscode";

/** Prompts for a non-empty FPAS expression, returning undefined on cancellation. */
export async function promptExpression(label: string, value: string): Promise<string | undefined> {
  return vscode.window.showInputBox({
    prompt: label,
    value,
    ignoreFocusOut: true,
    validateInput: input => input.trim().length === 0 ? "Enter one FPAS expression." : undefined
  });
}
