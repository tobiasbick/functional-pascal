import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import * as vscode from "vscode";

import type { FunctionalPascalExtensionApi } from "../../src/extension";

const CHECK_COMMAND = "functionalPascal.checkProject";
const BUILD_COMMAND = "functionalPascal.buildProject";
const FORMAT_COMMAND = "functionalPascal.formatProject";
const FORMAT_CHECK_COMMAND = "functionalPascal.checkProjectFormatting";
const RUN_COMMAND = "functionalPascal.runProject";

/** Verifies project workflows against the real bundled-development CLI. */
export async function verifyWorkflowHost(
  api: FunctionalPascalExtensionApi
): Promise<void> {
  const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  assert.ok(workspaceRoot, "extension test workspace is open");
  assert.equal(
    path.basename(api.workflow.cliPath()),
    process.platform === "win32" ? "fpas.exe" : "fpas"
  );
  const registered = await vscode.commands.getCommands(true);
  for (const command of api.workflow.commands) {
    assert.ok(registered.includes(command), `workflow command ${command} is registered`);
  }

  const fixtureRoot = path.join(
    workspaceRoot,
    `.workflow with spaces ${process.pid} ${Date.now()}`
  );
  const programRoot = path.join(fixtureRoot, "program");
  const testsRoot = path.join(fixtureRoot, "tests");
  const invalidRoot = path.join(fixtureRoot, "invalid");
  const programManifest = path.join(programRoot, "workflow.fpasprj");
  const testsManifest = path.join(testsRoot, "workflow-tests.fpasprj");
  const invalidManifest = path.join(invalidRoot, "invalid.fpasprj");
  const mainPath = path.join(programRoot, "src", "main.fpas");
  const testPaths = Object.fromEntries(
    ["pass", "fail", "skip", "compile", "runtime", "timeout"].map((name) => [
      name,
      path.join(testsRoot, `${name}_test.fpas`)
    ])
  );
  try {
    await fs.mkdir(path.dirname(mainPath), { recursive: true });
    await fs.mkdir(testsRoot, { recursive: true });
    await fs.mkdir(invalidRoot, { recursive: true });
    await fs.writeFile(
      programManifest,
      '[project]\nname = "workflow"\nkind = "program"\nmain = "src/main.fpas"\n\n[sources]\ninclude = ["src/**/*.fpas"]\n'
    );
    await fs.writeFile(
      mainPath,
      "program Workflow; begin var Value:integer:=MissingCall() end."
    );
    await fs.writeFile(
      testsManifest,
      '[project]\nname = "workflow-tests"\nkind = "test"\n\n[sources]\ninclude = ["*_test.fpas"]\n'
    );
    await fs.writeFile(
      testPaths.pass,
      "program PassTest;\n\nuses Std.Test;\n\nbegin\n  AssertTrue(true)\nend.\n"
    );
    await fs.writeFile(
      testPaths.fail,
      "program FailTest;\n\nuses Std.Test;\n\nbegin\n  AssertTrue(false)\nend.\n"
    );
    await fs.writeFile(
      testPaths.skip,
      "program SkipTest;\n\nuses Std.Test;\n\nbegin\n  Skip('host fixture')\nend.\n"
    );
    await fs.writeFile(
      testPaths.compile,
      "program CompileTest;\n\nbegin\n  var Value: MissingType := 1\nend.\n"
    );
    await fs.writeFile(
      testPaths.runtime,
      "program RuntimeTest;\n\nbegin\n  panic('host fixture')\nend.\n"
    );
    await fs.writeFile(
      testPaths.timeout,
      "program TimeoutTest;\n\nbegin\n  mutable var Value: integer := 0;\n  while true do\n  begin\n    Value := Value + 1\n  end\nend.\n"
    );
    await fs.writeFile(invalidManifest, "not valid toml");

    const programUri = vscode.Uri.file(programManifest);
    await api.workflow.selectProject(programUri);
    await vscode.commands.executeCommand(CHECK_COMMAND, programUri);
    const problems = vscode.languages.getDiagnostics(vscode.Uri.file(mainPath));
    assert.ok(
      problems.some(
        (diagnostic) =>
          diagnostic.source === "fpas workflow" &&
          diagnostic.code === "F2003" &&
          diagnostic.severity === vscode.DiagnosticSeverity.Error &&
          diagnostic.range.start.line === 0 &&
          diagnostic.range.start.character === 43 &&
          diagnostic.message.includes("Help:")
      ),
      JSON.stringify({ problems, operation: api.workflow.lastOperation() })
    );
    await vscode.commands.executeCommand(BUILD_COMMAND, programUri);
    assert.ok(
      vscode.languages.getDiagnostics(vscode.Uri.file(mainPath)).length > 0,
      JSON.stringify(api.workflow.lastOperation())
    );

    await vscode.commands.executeCommand(FORMAT_CHECK_COMMAND, programUri);
    await vscode.commands.executeCommand(FORMAT_COMMAND, programUri);
    assert.equal(
      await fs.readFile(mainPath, "utf8"),
      "program Workflow;\n\nbegin\n  var Value: integer := MissingCall()\nend.\n"
    );

    const terminalsBefore = vscode.window.terminals.length;
    await vscode.commands.executeCommand(RUN_COMMAND, {
      target: programUri,
      programArguments: ["one", "two words"]
    });
    assert.equal(vscode.window.terminals.length, terminalsBefore + 1);
    vscode.window.terminals.at(-1)?.dispose();

    await vscode.commands.executeCommand(CHECK_COMMAND, vscode.Uri.file(invalidManifest));
    assert.equal(api.languageServerStarted, true);

    await vscode.workspace
      .getConfiguration("functionalPascal")
      .update("testTimeoutSeconds", 1, vscode.ConfigurationTarget.Workspace);
    const testsUri = vscode.Uri.file(testsManifest);
    await api.workflow.selectProject(testsUri);
    const discovered = await api.workflow.discoverTests();
    assert.deepEqual(
      discovered.map((file) => path.basename(file)).sort(),
      [
        "compile_test.fpas",
        "fail_test.fpas",
        "pass_test.fpas",
        "runtime_test.fpas",
        "skip_test.fpas",
        "timeout_test.fpas"
      ]
    );
    const statuses = await api.workflow.runTests([
      testPaths.pass,
      testPaths.fail,
      testPaths.skip,
      testPaths.runtime,
      testPaths.timeout,
      testPaths.compile
    ]);
    assert.equal(statuses[testPaths.pass], "pass", JSON.stringify(statuses));
    assert.equal(statuses[testPaths.fail], "assert_failed");
    assert.equal(statuses[testPaths.skip], "skipped");
    assert.equal(statuses[testPaths.compile], "compile_error");
    assert.equal(statuses[testPaths.runtime], "runtime_error");
    assert.equal(statuses[testPaths.timeout], "timed_out");
    const compileProblems = vscode.languages.getDiagnostics(
      vscode.Uri.file(testPaths.compile)
    );
    assert.ok(
      compileProblems.some(
        (diagnostic) =>
          diagnostic.source === "fpas workflow" && diagnostic.code === "F2001"
      ),
      "test compilation diagnostics should remain available in Problems"
    );
  } finally {
    await vscode.workspace
      .getConfiguration("functionalPascal")
      .update("testTimeoutSeconds", undefined, vscode.ConfigurationTarget.Workspace);
    await fs.rm(fixtureRoot, { recursive: true, force: true });
  }
}
