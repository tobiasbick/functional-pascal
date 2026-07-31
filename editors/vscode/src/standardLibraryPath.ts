import { statSync } from "node:fs";
import path from "node:path";

import * as vscode from "vscode";

/** Finds the development or packaged source standard library. */
export function resolveStandardLibraryPath(
  context: vscode.ExtensionContext
): string {
  const root =
    context.extensionMode !== vscode.ExtensionMode.Production
      ? path.resolve(context.extensionPath, "..", "..", "lib")
      : path.join(context.extensionPath, "standard-library");
  const manifest = path.join(root, "stdlib.fpasprj");

  let isFile = false;
  try {
    isFile = statSync(manifest).isFile();
  } catch {
    isFile = false;
  }
  if (!isFile) {
    const recovery =
      context.extensionMode === vscode.ExtensionMode.Production
        ? "Rebuild the host-native VSIX and reinstall it."
        : "Restore `lib/stdlib.fpasprj` in the repository and restart the extension.";
    throw new Error(
      `Functional Pascal source standard library was not found at ${manifest}. ${recovery}`
    );
  }
  return root;
}
