import path from "node:path";

import * as vscode from "vscode";

import type { ParsedWorkflowDiagnostic } from "./model";

const DIAGNOSTIC_HEADER =
  /^(.*):(\d+):(\d+): (error|warning)\[(F\d{4})\]: (.*)$/u;
const WRAPPED_BUILD_ERROR = /^Cannot build [^`]+`([^`]+)`: (.*)$/u;
const PATHLESS_DIAGNOSTIC =
  /^(\d+):(\d+): (error|warning)\[(F\d{4})\]: (.*)$/u;

/** Parses stable compiler diagnostics while preserving optional help lines. */
export function parseWorkflowDiagnostics(
  stderr: string,
  cwd: string
): ParsedWorkflowDiagnostic[] {
  const lines = stderr.replaceAll("\r\n", "\n").split("\n");
  const diagnostics: ParsedWorkflowDiagnostic[] = [];
  for (let index = 0; index < lines.length; index += 1) {
    const lineText = lines[index].trimStart();
    const wrapped = WRAPPED_BUILD_ERROR.exec(lineText);
    const candidate = wrapped?.[2] ?? lineText;
    const match = DIAGNOSTIC_HEADER.exec(candidate);
    const pathless = match === null ? PATHLESS_DIAGNOSTIC.exec(candidate) : null;
    if (match === null && pathless === null) {
      continue;
    }
    const sourcePath = match?.[1] ?? wrapped?.[1];
    if (sourcePath === undefined) {
      continue;
    }
    const values = match ?? pathless!;
    const offset = match === null ? 0 : 1;
    const line = values[offset + 1];
    const column = values[offset + 2];
    const severity = values[offset + 3];
    const code = values[offset + 4];
    const message = values[offset + 5];
    const next = lines[index + 1]?.trimStart();
    const help = next?.startsWith("help: ")
      ? next.slice("help: ".length)
      : undefined;
    if (help !== undefined) {
      index += 1;
    }
    diagnostics.push({
      path: path.isAbsolute(sourcePath)
        ? path.normalize(sourcePath)
        : path.resolve(cwd, sourcePath),
      line: Math.max(0, Number.parseInt(line, 10) - 1),
      column: Math.max(0, Number.parseInt(column, 10) - 1),
      severity: severity as "error" | "warning",
      code,
      message,
      help
    });
  }
  return diagnostics;
}

/** Publishes workflow diagnostics under their real source URIs. */
export async function publishWorkflowDiagnostics(
  collection: vscode.DiagnosticCollection,
  diagnostics: readonly ParsedWorkflowDiagnostic[]
): Promise<void> {
  collection.clear();
  const grouped = new Map<string, ParsedWorkflowDiagnostic[]>();
  for (const diagnostic of diagnostics) {
    const values = grouped.get(diagnostic.path) ?? [];
    values.push(diagnostic);
    grouped.set(diagnostic.path, values);
  }
  for (const [sourcePath, values] of grouped) {
    const uri = vscode.Uri.file(sourcePath);
    const document = await Promise.resolve(vscode.workspace.openTextDocument(uri)).catch(
      () => undefined
    );
    const converted = values.map((value) => {
      const start = new vscode.Position(value.line, value.column);
      const range =
        document?.getWordRangeAtPosition(start) ??
        new vscode.Range(start, start.translate(0, 1));
      const message =
        value.help === undefined
          ? value.message
          : `${value.message}\n\nHelp: ${value.help}`;
      const diagnostic = new vscode.Diagnostic(
        range,
        message,
        value.severity === "warning"
          ? vscode.DiagnosticSeverity.Warning
          : vscode.DiagnosticSeverity.Error
      );
      diagnostic.code = value.code;
      diagnostic.source = "fpas workflow";
      return diagnostic;
    });
    collection.set(uri, converted);
  }
}

/** Parses and validates the versioned JSON test-report contract. */
export function parseTestReport(stdout: string): import("./model").WorkflowTestReport {
  const value: unknown = JSON.parse(stdout);
  if (!isRecord(value) || value.version !== 1 || !Array.isArray(value.tests)) {
    throw new Error("Unsupported Functional Pascal test report.");
  }
  const statuses = new Set([
    "pass",
    "skipped",
    "not_run",
    "assert_failed",
    "compile_error",
    "runtime_error",
    "timed_out"
  ]);
  const tests = value.tests.map((test) => {
    if (
      !isRecord(test) ||
      typeof test.file !== "string" ||
      typeof test.status !== "string" ||
      !statuses.has(test.status)
    ) {
      throw new Error("Malformed Functional Pascal test case report.");
    }
    return {
      file: test.file,
      status: test.status as import("./model").WorkflowTestStatus
    };
  });
  return { version: 1, tests };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
