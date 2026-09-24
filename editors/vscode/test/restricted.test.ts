import assert from "node:assert/strict";
import path from "node:path";

import * as vscode from "vscode";

/** Verifies that executable FPAS tooling remains disabled in Restricted Mode. */
export async function run(): Promise<void> {
  assert.equal(vscode.workspace.isTrusted, false, "fresh workspace is in Restricted Mode");
  const extension = vscode.extensions.getExtension("functional-pascal.functional-pascal");
  assert.ok(extension);
  const source = vscode.Uri.file(path.join(extension.extensionPath, "test", "fixtures",
    "standalone", "malformed_syntax.fpas"));
  await vscode.workspace.openTextDocument(source);
  assert.equal(extension.isActive, false, "FPAS executable tooling must not activate");
  console.log("Functional Pascal Restricted Mode policy test passed.");
}
