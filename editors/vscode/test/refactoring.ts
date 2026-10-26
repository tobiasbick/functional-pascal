import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import * as vscode from "vscode";

/** Applies normal editor refactorings to unsaved buffers through the registered LSP provider. */
export async function verifyRefactoring(): Promise<void> {
  const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  assert.ok(workspaceRoot);
  const root = path.join(workspaceRoot, `.refactoring-${process.pid}-${Date.now()}`);
  try {
    await fs.mkdir(root, { recursive: true });
    await fs.writeFile(path.join(root, "app.fpasprj"),
      '[project]\nname = "refactoring"\nkind = "program"\nmain = "main.fpas"\n[sources]\ninclude = ["*.fpas"]\n');
    for (const name of ["A", "Z", "Unused"]) {
      await fs.writeFile(path.join(root, `${name}.fpas`),
        `unit Host.${name};\npublic function Value(): integer;\nbegin\n  return 1;\nend function;\nend unit;\n`);
    }
    const source = "program HostRefactoring;\n\nuses Host.Z as Zed, Host.A as Alpha, Host.Unused as Unused;\n\nbegin\n  const Value := (Alpha.Value() + Zed.Value()) * 3;\nend.\n";
    const uri = vscode.Uri.file(path.join(root, "main.fpas"));
    await fs.writeFile(uri.fsPath, source);
    const document = await vscode.workspace.openTextDocument(uri);
    await vscode.window.showTextDocument(document);
    const organize = await action(uri, new vscode.Range(0, 0, 0, 0), vscode.CodeActionKind.SourceOrganizeImports);
    assert.ok(organize.edit);
    assert.equal(await vscode.workspace.applyEdit(organize.edit), true);
    assert.match(document.getText(), /uses Host.A as Alpha, Host.Z as Zed;/);
    assert.doesNotMatch(document.getText(), /Host.Unused/);
    assert.equal(document.isDirty, true, "organization updates the unsaved editor buffer");

    const selected = "(Alpha.Value() + Zed.Value())";
    const start = document.getText().indexOf(selected);
    assert.ok(start >= 0);
    const range = new vscode.Range(document.positionAt(start), document.positionAt(start+selected.length));
    const extract = await action(uri, range, vscode.CodeActionKind.RefactorExtract);
    assert.ok(extract.edit);
    assert.equal(await vscode.workspace.applyEdit(extract.edit), true);
    assert.match(document.getText(), /const ExtractedValue := \(Alpha.Value\(\) \+ Zed.Value\(\)\);/);
    assert.match(document.getText(), /const Value := ExtractedValue \* 3;/);
    assert.equal(await fs.readFile(uri.fsPath, "utf8"), source, "refactorings do not save the source implicitly");
  } finally {
    await vscode.commands.executeCommand("workbench.action.revertAndCloseActiveEditor");
    await fs.rm(root, { recursive: true, force: true });
  }
}

async function action(uri: vscode.Uri, range: vscode.Range, kind: vscode.CodeActionKind): Promise<vscode.CodeAction> {
  const deadline = Date.now()+10_000;
  while (Date.now()<deadline) {
    const actions = await vscode.commands.executeCommand<Array<vscode.CodeAction | vscode.Command>>(
      "vscode.executeCodeActionProvider", uri, range, kind.value);
    const result = actions?.find((candidate): candidate is vscode.CodeAction =>
      "edit" in candidate && candidate.kind?.value === kind.value);
    if (result) return result;
    await new Promise(resolve => setTimeout(resolve, 50));
  }
  assert.fail(`no ${kind.value} code action`);
}
