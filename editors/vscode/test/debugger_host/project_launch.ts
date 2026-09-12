/** Extension Host regression coverage for zero-configuration project launch targets. */

import assert from "node:assert/strict";
import path from "node:path";

import * as vscode from "vscode";

import { FunctionalPascalDebugConfigurationProvider } from "../../src/debugger/adapter";

/** Verify that F5 resolves project sources and manifests to executable project targets. */
export async function verifyProjectLaunchTargets(workspaceRoot: string): Promise<void> {
  const project = path.join(
    workspaceRoot,
    "workspace",
    "apps",
    "demo",
    "demo.fpasprj"
  );
  const main = path.join(path.dirname(project), "src", "main.fpas");
  const workspace = path.join(workspaceRoot, "workspace", "phase1.fpasworkspace");
  const provider = new FunctionalPascalDebugConfigurationProvider();

  const mainDocument = await vscode.workspace.openTextDocument(main);
  await vscode.window.showTextDocument(mainDocument);
  const mainConfiguration = await provider.resolveDebugConfiguration(
    undefined,
    {} as vscode.DebugConfiguration
  );
  assert.equal(mainConfiguration?.program, project);

  const projectDocument = await vscode.workspace.openTextDocument(project);
  assert.equal(projectDocument.languageId, "fpas-project");
  await vscode.window.showTextDocument(projectDocument);
  const projectConfiguration = await provider.resolveDebugConfiguration(
    undefined,
    {} as vscode.DebugConfiguration
  );
  assert.equal(projectConfiguration?.program, project);

  const workspaceDocument = await vscode.workspace.openTextDocument(workspace);
  assert.equal(workspaceDocument.languageId, "fpas-project");
  await vscode.window.showTextDocument(workspaceDocument);
  const workspaceConfiguration = await provider.resolveDebugConfiguration(
    undefined,
    {} as vscode.DebugConfiguration
  );
  assert.equal(workspaceConfiguration?.program, workspace);

  await vscode.commands.executeCommand("workbench.action.closeAllEditors");
}
