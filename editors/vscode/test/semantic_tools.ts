import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import * as vscode from "vscode";

const DIAGNOSTIC_CODE = "F2003";

interface SemanticTokensLegendResult {
  readonly tokenTypes: readonly string[];
  readonly tokenModifiers: readonly string[];
}

/** Verifies semantic highlighting and a diagnostic-bound quick fix in VS Code. */
export async function verifySemanticTools(
  extension: vscode.Extension<unknown>
): Promise<void> {
  const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  assert.ok(workspaceRoot, "extension test workspace is open");
  assert.ok(
    extension.packageJSON.contributes.grammars.some(
      (grammar: { language?: string }) => grammar.language === "fpas"
    ),
    "the TextMate fallback remains contributed"
  );

  const fixtureRoot = path.join(
    workspaceRoot,
    `.semantic-tools-${process.pid}-${Date.now()}`
  );
  const sourcePath = path.join(fixtureRoot, "src", "main.fpas");
  const source =
    "program SemanticHost;\n\nuses Semantic.Core;\n\nbegin\n  var Music: string := '𝄞' + ExistingText;\n  var Value: integer := UniqueValue()\nend.\n";
  let document: vscode.TextDocument | undefined;
  try {
    await fs.mkdir(path.dirname(sourcePath), { recursive: true });
    await fs.writeFile(
      path.join(fixtureRoot, "semantic-host.fpasprj"),
      '[project]\nname = "semantic-host"\nkind = "program"\nmain = "src/main.fpas"\n\n[sources]\ninclude = ["src/**/*.fpas"]\n'
    );
    await fs.writeFile(
      path.join(fixtureRoot, "src", "core.fpas"),
      "unit Semantic.Core;\n\npublic const ExistingText: string := 'ok';\n"
    );
    await fs.writeFile(
      path.join(fixtureRoot, "src", "importable.fpas"),
      "unit Semantic.Importable;\n\npublic function UniqueValue(): integer;\nbegin\n  return 42\nend;\n"
    );
    await fs.writeFile(sourcePath, source);

    const uri = vscode.Uri.file(sourcePath);
    document = await vscode.workspace.openTextDocument(uri);
    await vscode.window.showTextDocument(document);
    const diagnostics = await waitForDiagnostics(uri, (values) =>
      values.some((diagnostic) => diagnostic.code === DIAGNOSTIC_CODE)
    );
    const unknown = diagnostics.find(
      (diagnostic) => diagnostic.code === DIAGNOSTIC_CODE
    );
    assert.ok(unknown, "unknown-name diagnostic is available");

    const legend = await vscode.commands.executeCommand<SemanticTokensLegendResult>(
      "vscode.provideDocumentSemanticTokensLegend",
      uri
    );
    assert.ok(legend, "semantic token legend is available");
    assert.ok(legend.tokenTypes.includes("constant"));
    assert.deepEqual(legend.tokenModifiers, ["declaration", "readonly", "public"]);

    const tokens = await waitForSemanticTokens(uri);
    assert.equal(tokens.data.length % 5, 0);
    assert.ok(tokens.data.length > 0, "semantic token data is available");
    const constantType = legend.tokenTypes.indexOf("constant");
    assert.ok(
      Array.from(tokens.data).some(
        (value, index) => index % 5 === 3 && value === constantType
      ),
      "the public constant reference has a semantic token"
    );

    const action = await waitForCodeAction(uri, unknown.range);
    assert.equal(action.kind?.value, vscode.CodeActionKind.QuickFix.value);
    assert.equal(action.isPreferred, true);
    assert.ok(action.edit, "quick fix has a workspace edit");
    assert.equal(await vscode.workspace.applyEdit(action.edit), true);
    assert.match(
      document.getText(),
      /uses Semantic\.Core, Semantic\.Importable;/
    );
    await waitForDiagnostics(
      uri,
      (values) => !values.some((diagnostic) => diagnostic.code === DIAGNOSTIC_CODE)
    );
  } finally {
    if (document && !document.isClosed) {
      await vscode.commands.executeCommand(
        "workbench.action.revertAndCloseActiveEditor"
      );
    }
    await fs.rm(fixtureRoot, { recursive: true, force: true });
  }
}

async function waitForSemanticTokens(
  uri: vscode.Uri
): Promise<vscode.SemanticTokens> {
  const deadline = Date.now() + 10_000;
  while (Date.now() < deadline) {
    const tokens = await vscode.commands.executeCommand<vscode.SemanticTokens>(
      "vscode.provideDocumentSemanticTokens",
      uri
    );
    if (tokens?.data.length) {
      return tokens;
    }
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  assert.fail("timed out waiting for semantic tokens");
}

async function waitForCodeAction(
  uri: vscode.Uri,
  range: vscode.Range
): Promise<vscode.CodeAction> {
  const deadline = Date.now() + 10_000;
  while (Date.now() < deadline) {
    const actions =
      (await vscode.commands.executeCommand<
        Array<vscode.CodeAction | vscode.Command>
      >(
        "vscode.executeCodeActionProvider",
        uri,
        range,
        vscode.CodeActionKind.QuickFix.value
      )) ?? [];
    const action = actions.find(
      (candidate): candidate is vscode.CodeAction =>
        candidate instanceof vscode.CodeAction &&
        candidate.title === "Import Semantic.Importable"
    );
    if (action) {
      return action;
    }
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  assert.fail("timed out waiting for the Functional Pascal import quick fix");
}

async function waitForDiagnostics(
  uri: vscode.Uri,
  predicate: (diagnostics: readonly vscode.Diagnostic[]) => boolean
): Promise<readonly vscode.Diagnostic[]> {
  const deadline = Date.now() + 10_000;
  while (Date.now() < deadline) {
    const diagnostics = vscode.languages.getDiagnostics(uri);
    if (predicate(diagnostics)) {
      return diagnostics;
    }
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  const diagnostics = vscode.languages.getDiagnostics(uri);
  assert.fail(`timed out waiting for diagnostics: ${JSON.stringify(diagnostics)}`);
}
