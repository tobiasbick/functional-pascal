import { statSync } from "node:fs";
import path from "node:path";
import process from "node:process";

import * as vscode from "vscode";

const REMOTE_HOST_MESSAGE =
  "Functional Pascal language-server support is unavailable in remote extension hosts. Open the workspace in a local desktop editor; remote SSH, WSL, and container extension hosts are not supported by this hobby-project build.";

/** Resolves a supported VS Code host target name. */
export function resolveHostTarget(
  platform: NodeJS.Platform,
  architecture: string
): string {
  const target = `${platform}-${architecture}`;
  if (
    ![
      "win32-x64",
      "win32-arm64",
      "linux-x64",
      "linux-arm64",
      "darwin-x64",
      "darwin-arm64"
    ].includes(target)
  ) {
    throw new Error(
      `Unsupported Functional Pascal VSIX host target: ${target}. Build on Windows, Linux, or macOS using an x64 or arm64 host.`
    );
  }
  return target;
}

/** Finds the development or packaged server without consulting the system PATH. */
export function resolveServerPath(context: vscode.ExtensionContext): string {
  if (vscode.env.remoteName !== undefined) {
    throw new Error(REMOTE_HOST_MESSAGE);
  }

  const executable = process.platform === "win32" ? "fpas-lsp.exe" : "fpas-lsp";
  const candidate =
    context.extensionMode !== vscode.ExtensionMode.Production
      ? path.resolve(
          context.extensionPath,
          "..",
          "..",
          "target",
          "debug",
          executable
        )
      : path.join(
          context.extensionPath,
          "server",
          resolveHostTarget(process.platform, process.arch),
          executable
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
        : "Build it with `cargo build -p fpas-lsp` and restart the extension.";
    throw new Error(
      `Functional Pascal language server was not found at ${candidate}. ${recovery}`
    );
  }
  return candidate;
}
