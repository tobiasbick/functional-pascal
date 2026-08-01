import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import * as vscode from "vscode";

import {
  cliCandidatePath,
  cliExecutableName,
  resolveCliPath
} from "../../src/cliPath";
import {
  operationArguments,
  runArguments,
  testRunArguments
} from "../../src/workflow/arguments";
import {
  parseTestReport,
  parseWorkflowDiagnostics
} from "../../src/workflow/diagnostics";
import { parseProgramArguments } from "../../src/workflow/controller";
import {
  CliCompatibility,
  parseCliVersion
} from "../../src/workflow/cliCompatibility";
import { WorkflowProcessRunner } from "../../src/workflow/processes";
import { rememberedProject } from "../../src/workflow/project";

/** Runs pure workflow contract and cancellation tests. */
export async function verifyWorkflowUnits(): Promise<void> {
  const nativeRoot = path.parse(process.cwd()).root;
  const target = path.join(nativeRoot, "workspace with spaces", "demo.fpasprj");
  const library = path.join(nativeRoot, "extension with spaces", "standard-library");
  assert.deepEqual(operationArguments("check", target, library), [
    "check",
    "--std-lib",
    library,
    target
  ]);
  assert.deepEqual(operationArguments("formatCheck", target, library), [
    "fmt",
    "--check",
    target
  ]);
  assert.deepEqual(testRunArguments(target, library, "one test", 3), [
    "test",
    "--std-lib",
    library,
    "--report",
    "json",
    "--timeout",
    "3",
    "--filter",
    "one test",
    target
  ]);
  assert.deepEqual(runArguments(target, library, ["one", "two words"]), [
    "run",
    "--std-lib",
    library,
    target,
    "--",
    "one",
    "two words"
  ]);
  assert.deepEqual(parseProgramArguments('["one", "two words"]'), [
    "one",
    "two words"
  ]);
  assert.throws(() => parseProgramArguments("[1]"), /array of strings/u);

  const candidates = ["C:\\work\\one.fpasprj", "C:\\work\\two.fpasprj"];
  assert.equal(
    rememberedProject(candidates, "c:\\WORK\\TWO.fpasprj"),
    candidates[1]
  );
  assert.equal(rememberedProject(candidates, "C:\\work\\gone.fpasprj"), undefined);

  const diagnosticRoot = path.join(nativeRoot, "project with spaces");
  const mainDiagnostic = path.join(diagnosticRoot, "main.fpas");
  const wrappedDiagnostic = path.join(diagnosticRoot, "wrapped.fpas");
  const parsed = parseWorkflowDiagnostics(
    `${mainDiagnostic}:12:8: error[F2003]: Unknown function \`Missing\`\n  help: Add a declaration.\nrelative.fpas:2:3: warning[F2004]: Warning text\nCannot build project \`${wrappedDiagnostic}\`: 3:4: error[F2003]: Wrapped error\n        ${mainDiagnostic}:5:6: error[F2001]: Indented test error\n          help: Add the missing type.\n`,
    diagnosticRoot
  );
  assert.equal(parsed.length, 4);
  assert.equal(parsed[0].path, path.normalize(mainDiagnostic));
  assert.equal(parsed[0].line, 11);
  assert.equal(parsed[0].column, 7);
  assert.equal(parsed[0].code, "F2003");
  assert.equal(parsed[0].help, "Add a declaration.");
  assert.equal(parsed[1].severity, "warning");
  assert.equal(parsed[2].path, path.normalize(wrappedDiagnostic));
  assert.equal(parsed[2].line, 2);
  assert.equal(parsed[2].column, 3);
  assert.equal(parsed[3].path, path.normalize(mainDiagnostic));
  assert.equal(parsed[3].code, "F2001");
  assert.equal(parsed[3].help, "Add the missing type.");

  const statuses = [
    "pass",
    "skipped",
    "not_run",
    "assert_failed",
    "compile_error",
    "runtime_error",
    "timed_out"
  ];
  const report = parseTestReport(
    JSON.stringify({
      version: 1,
      tests: statuses.map((status) => ({ file: `${status}_test.fpas`, status }))
    })
  );
  assert.deepEqual(
    report.tests.map((test) => test.status),
    statuses
  );
  assert.throws(
    () => parseTestReport('{"version":2,"tests":[]}'),
    /Unsupported/u
  );

  assert.equal(cliExecutableName("win32"), "fpas.exe");
  assert.equal(cliExecutableName("linux"), "fpas");
  assert.equal(parseCliVersion("fpas 0.0.1\n"), "0.0.1");
  assert.equal(parseCliVersion("unexpected"), undefined);
  assert.match(
    cliCandidatePath("C:\\extension", vscode.ExtensionMode.Production, "win32", "x64"),
    /cli[\\/]win32-x64[\\/]fpas\.exe$/u
  );
  const missingRoot = await fs.mkdtemp(path.join(os.tmpdir(), "fpas-cli-missing-"));
  try {
    assert.throws(
      () =>
        resolveCliPath({
          extensionPath: path.join(missingRoot, "editors", "vscode"),
          extensionMode: vscode.ExtensionMode.Development
        } as vscode.ExtensionContext),
      /cargo build -p fpas-cli/u
    );
  } finally {
    await fs.rm(missingRoot, { recursive: true, force: true });
  }

  const output = { append: () => undefined } as unknown as vscode.LogOutputChannel;
  const runner = new WorkflowProcessRunner(output);
  const incompatible = new CliCompatibility(() => process.execPath, runner);
  await assert.rejects(() => incompatible.resolve(), /CLI is incompatible/u);
  const cancellation = new vscode.CancellationTokenSource();
  const pending = runner.run(
    process.execPath,
    ["-e", "console.log(process.pid); setInterval(() => {}, 1000)"],
    process.cwd(),
    cancellation.token
  );
  setTimeout(() => cancellation.cancel(), 100);
  const cancelled = await pending;
  cancellation.dispose();
  assert.equal(cancelled.cancelled, true);
  assert.notEqual(cancelled.exitCode, 0);
  const pid = Number.parseInt(cancelled.stdout.trim(), 10);
  assert.ok(Number.isInteger(pid));
  assert.throws(() => process.kill(pid, 0));
}
