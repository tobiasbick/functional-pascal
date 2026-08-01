import { spawn } from "node:child_process";

import * as vscode from "vscode";

import type { WorkflowProcessResult } from "./model";

/** Runs shell-free native CLI processes and owns cancellation cleanup. */
export class WorkflowProcessRunner {
  public constructor(private readonly output: vscode.LogOutputChannel) {}

  /** Runs one process and resolves only after it exits or is cancelled. */
  public run(
    executable: string,
    args: readonly string[],
    cwd: string,
    token?: vscode.CancellationToken
  ): Promise<WorkflowProcessResult> {
    return new Promise((resolve, reject) => {
      const child = spawn(executable, args, {
        cwd,
        shell: false,
        windowsHide: true,
        stdio: ["ignore", "pipe", "pipe"]
      });
      let stdout = "";
      let stderr = "";
      let cancelled = token?.isCancellationRequested ?? false;
      const cancellation = token?.onCancellationRequested(() => {
        cancelled = true;
        child.kill();
      });
      child.stdout.setEncoding("utf8");
      child.stderr.setEncoding("utf8");
      child.stdout.on("data", (chunk: string) => {
        stdout += chunk;
        this.output.append(chunk);
      });
      child.stderr.on("data", (chunk: string) => {
        stderr += chunk;
        this.output.append(chunk);
      });
      child.once("error", (error) => {
        cancellation?.dispose();
        reject(error);
      });
      child.once("close", (exitCode) => {
        cancellation?.dispose();
        resolve({ exitCode, stdout, stderr, cancelled });
      });
      if (cancelled) {
        child.kill();
      }
    });
  }
}
