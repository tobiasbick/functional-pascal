/** User-facing selection and recovery actions for the FPAS executable. */

import * as vscode from "vscode";

import { EXECUTABLE_PATH_SETTING } from "./cliPath";

/** Command that selects an installed Functional Pascal executable. */
export const SELECT_EXECUTABLE_COMMAND = "functionalPascal.selectExecutable";

/** Prompt for an executable and save it in machine-specific user settings. */
export async function selectExecutable(): Promise<string | undefined> {
  const selected = await vscode.window.showOpenDialog({
    canSelectFiles: true,
    canSelectFolders: false,
    canSelectMany: false,
    openLabel: "Select FPAS Executable",
    title: "Select the installed fpas executable"
  });
  const executable = selected?.[0]?.fsPath;
  if (executable === undefined) return undefined;
  await vscode.workspace
    .getConfiguration("functionalPascal")
    .update("executablePath", executable, vscode.ConfigurationTarget.Global);
  return executable;
}

/** Show one actionable toolchain error with selection and settings actions. */
export async function offerToolchainRecovery(error: unknown): Promise<void> {
  const message = error instanceof Error ? error.message : String(error);
  const choice = await vscode.window.showErrorMessage(
    message,
    "Select FPAS Executable",
    "Open Settings"
  );
  if (choice === "Select FPAS Executable") {
    await vscode.commands.executeCommand(SELECT_EXECUTABLE_COMMAND);
  } else if (choice === "Open Settings") {
    await vscode.commands.executeCommand(
      "workbench.action.openSettings",
      EXECUTABLE_PATH_SETTING
    );
  }
}
