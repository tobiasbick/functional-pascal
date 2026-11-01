import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import * as vscode from "vscode";
import { INSPECT_COMMAND } from "../src/sourceTools/inspect";

/** Exercises the registered command, dirty dependency data, read-only view and stale rejection. */
export async function verifySourceTools(): Promise<void> {
  assert.ok((await vscode.commands.getCommands(true)).includes(INSPECT_COMMAND));
  const workspace = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  assert.ok(workspace);
  const root = await fs.mkdtemp(path.join(workspace, ".source-tools-"));
  const saved = "unit Inspect.Api;\n\npublic function Answer(): integer;\nbegin\n  return 42;\nend function;\nend unit;\n";
  const documents: vscode.TextDocument[] = [];
  try {
    const manifest = path.join(root, "app.fpasprj");
    await fs.writeFile(manifest, "[project]\nname='inspect'\nkind='program'\nmain='main.fpas'\n[sources]\ninclude=['*.fpas']\n");
    await fs.writeFile(path.join(root, "api.fpas"), saved);
    await fs.writeFile(path.join(root, "main.fpas"), "program Main; uses Inspect.Api as Api; begin discard Api.Answer(); end.");
    const unit = await vscode.workspace.openTextDocument(path.join(root, "api.fpas"));
    const main = await vscode.workspace.openTextDocument(path.join(root, "main.fpas"));
    documents.push(unit, main);
    const edit = new vscode.WorkspaceEdit();
    edit.replace(unit.uri, new vscode.Range(unit.positionAt(0), unit.positionAt(saved.length)),
      saved.replace("public function", "// Unsaved documentation.\npublic function").replace("42", "73"));
    assert.equal(await vscode.workspace.applyEdit(edit), true);
    const editor = await vscode.window.showTextDocument(main);
    const position = main.positionAt(main.getText().indexOf("Answer"));
    editor.selection = new vscode.Selection(position, position);
    const view = await vscode.commands.executeCommand<vscode.TextDocument>(INSPECT_COMMAND);
    assert.ok(view, "registered command returns the visible inspection document");
    documents.push(view);
    assert.equal(view.uri.scheme, "fpas-inspect");
    assert.ok(view.getText().includes("Unsaved documentation."), view.getText());
    assert.ok(view.getText().includes("return 73;"), view.getText());
    assert.ok(view.getText().includes("## Implementation"));
    assert.equal(vscode.window.activeTextEditor?.document.uri.toString(), view.uri.toString());
    const before = view.getText();
    await vscode.commands.executeCommand("type", { text: "changed" });
    assert.equal(view.getText(), before, "inspection cannot be edited");
    const change = new vscode.WorkspaceEdit();
    change.insert(unit.uri, unit.positionAt(0), "// Changed dependency.\n");
    assert.equal(await vscode.workspace.applyEdit(change), true);
    await waitFor(() => view.getText().includes("Inspection is out of date"));
    assert.ok(!view.getText().includes("return 73;"), "stale content is withdrawn");
    const refreshed = await vscode.commands.executeCommand<vscode.TextDocument>(INSPECT_COMMAND,
      { uri: main.uri, position, context: vscode.Uri.file(manifest) });
    assert.ok(refreshed);
    await waitFor(() => refreshed.getText().includes("return 73;"));
    assert.equal(await fs.readFile(path.join(root, "api.fpas"), "utf8"), saved, "inspection never saves dirty dependencies");
  } finally {
    for (const document of documents.reverse()) {
      if (!document.isClosed) {
        await vscode.window.showTextDocument(document);
        await vscode.commands.executeCommand("workbench.action.revertAndCloseActiveEditor");
      }
    }
    await fs.rm(root, { recursive: true, force: true, maxRetries: 20, retryDelay: 100 });
  }
}

async function waitFor(predicate: () => boolean): Promise<void> {
  const deadline = Date.now() + 10_000;
  while (Date.now() < deadline) {
    if (predicate()) return;
    await new Promise(resolve => setTimeout(resolve, 30));
  }
  assert.fail("inspection view did not update");
}
