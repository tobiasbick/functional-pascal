/** Installed Functional Pascal executable discovery. */

import { accessSync, constants, statSync } from "node:fs";
import path from "node:path";
import process from "node:process";

import * as vscode from "vscode";

/** Machine-scoped setting that overrides PATH-based toolchain discovery. */
export const EXECUTABLE_PATH_SETTING = "functionalPascal.executablePath";

/** Native Functional Pascal CLI filename for one platform. */
export function cliExecutableName(platform: NodeJS.Platform): string {
  return platform === "win32" ? "fpas.exe" : "fpas";
}

/** Resolve the configured executable first and otherwise search PATH. */
export function resolveCliPath(): string {
  if (vscode.env.remoteName !== undefined) {
    throw new Error(
      "Functional Pascal tooling does not support remote workspaces. Open a local folder with FPAS installed in the same desktop environment."
    );
  }
  const configured = vscode.workspace
    .getConfiguration("functionalPascal")
    .get<string>("executablePath", "");
  return resolveCliCandidate(
    configured,
    process.env.PATH ?? "",
    process.platform,
    isExecutableFile
  );
}

/** Resolve an explicit candidate or one executable from a supplied search path. */
export function resolveCliCandidate(
  configured: string,
  searchPath: string,
  platform: NodeJS.Platform,
  isExecutable: (candidate: string) => boolean
): string {
  const selected = configured.trim();
  if (selected.length > 0) {
    if (!path.isAbsolute(selected)) {
      throw new Error(
        `Functional Pascal setting \`${EXECUTABLE_PATH_SETTING}\` must be an absolute path: ${selected}`
      );
    }
    if (!isExecutable(selected)) {
      throw new Error(`Configured Functional Pascal executable was not found: ${selected}`);
    }
    return path.normalize(selected);
  }

  const executable = cliExecutableName(platform);
  for (const directory of searchPath.split(path.delimiter)) {
    const normalized = unquote(directory.trim());
    if (normalized.length === 0) continue;
    const candidate = path.join(normalized, executable);
    if (isExecutable(candidate)) return path.normalize(candidate);
  }
  throw new Error(
    `Functional Pascal executable \`${executable}\` was not found on PATH. Set \`${EXECUTABLE_PATH_SETTING}\` to the installed executable.`
  );
}

function isExecutableFile(candidate: string): boolean {
  try {
    if (!statSync(candidate).isFile()) return false;
    accessSync(candidate, constants.X_OK);
    return true;
  } catch {
    return false;
  }
}

function unquote(value: string): string {
  return value.startsWith('"') && value.endsWith('"')
    ? value.slice(1, -1)
    : value;
}
