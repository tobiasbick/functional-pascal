import { statSync } from "node:fs";
import path from "node:path";
import process from "node:process";

import * as vscode from "vscode";

import { resolveHostTarget } from "./serverPath";

/** Returns the native Functional Pascal CLI filename for one platform. */
export function cliExecutableName(platform: NodeJS.Platform): string {
  return platform === "win32" ? "fpas.exe" : "fpas";
}

/** Returns the deterministic development or packaged CLI candidate. */
export function cliCandidatePath(
  extensionPath: string,
  extensionMode: vscode.ExtensionMode,
  platform: NodeJS.Platform,
  architecture: string
): string {
  const executable = cliExecutableName(platform);
  return extensionMode !== vscode.ExtensionMode.Production
    ? path.resolve(extensionPath, "..", "..", "target", "debug", executable)
    : path.join(
        extensionPath,
        "cli",
        resolveHostTarget(platform, architecture),
        executable
      );
}

/** Finds the bundled CLI without consulting the system PATH. */
export function resolveCliPath(context: vscode.ExtensionContext): string {
  if (vscode.env.remoteName !== undefined) {
    throw new Error(
      "Functional Pascal project workflows require a local desktop extension host. Remote SSH, WSL, and container hosts are not supported by this hobby-project build."
    );
  }
  const candidate = cliCandidatePath(
    context.extensionPath,
    context.extensionMode,
    process.platform,
    process.arch
  );
  let isFile = false;
  try {
    isFile = statSync(candidate).isFile();
  } catch {
    isFile = false;
  }
  if (!isFile) {
    const recovery =
      context.extensionMode === vscode.ExtensionMode.Production
        ? "Rebuild the host-native VSIX and reinstall it."
        : "Build it with `cargo build -p fpas-cli` and retry the command.";
    throw new Error(
      `Functional Pascal CLI was not found at ${candidate}. ${recovery}`
    );
  }
  return candidate;
}
