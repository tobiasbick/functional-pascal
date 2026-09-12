import * as vscode from "vscode";

/** Setting that selects where interactive FPAS programs run. */
export const PROGRAM_TERMINAL_SETTING = "functionalPascal.programTerminal";

/** Supported program terminal locations. */
export type ProgramTerminal = "integratedTerminal" | "externalTerminal";

/** Normalize a configured terminal value onto the supported default. */
export function resolveProgramTerminal(value: unknown): ProgramTerminal {
  return value === "externalTerminal" ? "externalTerminal" : "integratedTerminal";
}

/** Read the program terminal selected for a workspace resource. */
export function configuredProgramTerminal(resource?: vscode.Uri): ProgramTerminal {
  const value = vscode.workspace
    .getConfiguration("functionalPascal", resource)
    .get<unknown>("programTerminal");
  return resolveProgramTerminal(value);
}
