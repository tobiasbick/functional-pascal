import assert from "node:assert/strict";
import path from "node:path";
import * as vscode from "vscode";

import type { FunctionalPascalExtensionApi } from "../src/extension";

const EXTENSION_ID = "functional-pascal.functional-pascal";
const SHOW_OUTPUT_COMMAND = "functionalPascal.showOutput";
const RESTART_LANGUAGE_SERVER_COMMAND =
  "functionalPascal.restartLanguageServer";
const ACTIVATION_MESSAGE =
  "Functional Pascal extension activated (Hello World).";

/** Runs the extension-shell regression test in a real Extension Host. */
export async function run(): Promise<void> {
  const extension =
    vscode.extensions.getExtension<FunctionalPascalExtensionApi>(EXTENSION_ID);

  assert.ok(extension, `extension ${EXTENSION_ID} is available`);

  const api = await extension.activate();
  assert.equal(extension.isActive, true);
  assert.equal(api.activationMessage, ACTIVATION_MESSAGE);
  assert.equal(api.languageServerStarted, true, api.languageServerError);
  assert.equal(
    path.basename(api.languageServerPath ?? ""),
    process.platform === "win32" ? "fpas-lsp.exe" : "fpas-lsp"
  );

  const commands = await vscode.commands.getCommands(true);
  assert.ok(commands.includes(SHOW_OUTPUT_COMMAND));
  assert.ok(commands.includes(RESTART_LANGUAGE_SERVER_COMMAND));

  const fixture = vscode.Uri.file(
    path.join(extension.extensionPath, "test", "grammar", "positive.fpas")
  );
  const document = await vscode.workspace.openTextDocument(fixture);
  assert.equal(document.languageId, "fpas");
  await vscode.window.showTextDocument(document);

  await vscode.commands.executeCommand(RESTART_LANGUAGE_SERVER_COMMAND);
  await vscode.commands.executeCommand(SHOW_OUTPUT_COMMAND);
  console.log("Functional Pascal extension and LSP lifecycle test passed.");
}
