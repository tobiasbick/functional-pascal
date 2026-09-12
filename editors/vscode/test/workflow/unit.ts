import assert from "node:assert/strict";
import path from "node:path";
import process from "node:process";
import * as vscode from "vscode";

import {
  cliExecutableName,
  resolveCliCandidate
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
import { externalTerminalInvocation } from "../../src/workflow/externalTerminal";
import { parseToolchainEnvironment } from "../../src/toolchain";
import { resolveProgramTerminal } from "../../src/programTerminal";
import { WorkflowProcessRunner } from "../../src/workflow/processes";
import { rememberedProject } from "../../src/workflow/project";

/** Runs pure workflow contract and cancellation tests. */
export async function verifyWorkflowUnits(): Promise<void> {
  const nativeRoot = path.parse(process.cwd()).root;
  const target = path.join(nativeRoot, "workspace with spaces", "demo.fpasprj");
  assert.deepEqual(operationArguments("check", target), ["check", target]);
  assert.deepEqual(operationArguments("formatCheck", target), [
    "fmt",
    "--check",
    target
  ]);
  assert.deepEqual(testRunArguments(target, "one test", 3), [
    "test",
    "--report",
    "json",
    "--timeout",
    "3",
    "--filter",
    "one test",
    target
  ]);
  assert.deepEqual(runArguments(target, ["one", "two words"]), [
    "run",
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
  assert.equal(resolveProgramTerminal(undefined), "integratedTerminal");
  assert.equal(resolveProgramTerminal("externalTerminal"), "externalTerminal");
  const windowsTerminal = externalTerminalInvocation(
    "win32",
    "C:\\FPAS\\fpas.exe",
    ["run", target],
    nativeRoot
  );
  assert.match(windowsTerminal.command, /cmd\.exe$/iu);
  assert.deepEqual(windowsTerminal.args.slice(0, 5), ["/d", "/s", "/c", "start", ""]);
  assert.equal(windowsTerminal.args.at(-2), "-EncodedCommand");
  assert.deepEqual(
    externalTerminalInvocation("linux", "/opt/fpas", ["run", target], "/work", "/usr/bin/xterm"),
    { command: "/usr/bin/xterm", args: ["-e", "/opt/fpas", "run", target] }
  );

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
  const configuredExecutable = path.join(nativeRoot, "FPAS", "fpas.exe");
  assert.equal(
    resolveCliCandidate(
      configuredExecutable,
      "",
      "win32",
      (candidate) => candidate === configuredExecutable
    ),
    path.normalize(configuredExecutable)
  );
  const pathDirectory = path.join(nativeRoot, "Tools", "FPAS");
  assert.equal(
    resolveCliCandidate("", pathDirectory, "win32", (candidate) =>
      candidate === path.join(pathDirectory, "fpas.exe")
    ),
    path.join(pathDirectory, "fpas.exe")
  );
  assert.throws(
    () => resolveCliCandidate("", "", "win32", () => false),
    /was not found on PATH/u
  );
  assert.throws(
    () => resolveCliCandidate("relative/fpas.exe", "", "win32", () => true),
    /absolute path/u
  );
  assert.deepEqual(
    parseToolchainEnvironment(
      configuredExecutable,
      JSON.stringify({
        schemaVersion: 1,
        version: "0.0.1",
        executable: configuredExecutable,
        standardLibrary: path.join(nativeRoot, "FPAS", "lib")
      }),
      () => true
    ),
    {
      executable: configuredExecutable,
      standardLibrary: path.join(nativeRoot, "FPAS", "lib"),
      version: "0.0.1"
    }
  );
  assert.throws(
    () =>
      parseToolchainEnvironment(
        configuredExecutable,
        JSON.stringify({
          schemaVersion: 1,
          version: "9.9.9",
          executable: configuredExecutable,
          standardLibrary: path.join(nativeRoot, "FPAS", "lib")
        }),
        () => true
      ),
    /Expected version 0\.0\.1, received 9\.9\.9/u
  );
  assert.throws(
    () => parseToolchainEnvironment(configuredExecutable, "not JSON", () => true),
    /invalid JSON/u
  );

  const output = { append: () => undefined } as unknown as vscode.LogOutputChannel;
  const runner = new WorkflowProcessRunner(output);
  const cancellation = new vscode.CancellationTokenSource();
  const pending = runner.run(
    process.execPath,
    ["-e", "console.log(process.pid); setInterval(() => {}, 1000)"],
    process.cwd(),
    cancellation.token
  );
  setTimeout(() => cancellation.cancel(), 500);
  const cancelled = await pending;
  cancellation.dispose();
  assert.equal(cancelled.cancelled, true);
  assert.notEqual(cancelled.exitCode, 0);
  const pid = Number.parseInt(cancelled.stdout.trim(), 10);
  assert.ok(Number.isInteger(pid));
  assert.throws(() => process.kill(pid, 0));
}
