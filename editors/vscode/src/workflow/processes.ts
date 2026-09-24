import { spawn } from "node:child_process";

import * as vscode from "vscode";

import type { WorkflowProcessResult } from "./model";

/** Reports and diagnostics are complete or explicitly rejected, never silently truncated. */
const MAX_CAPTURE_BYTES = 16 * 1024 * 1024;

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
      let stdoutBytes = 0;
      let stderrBytes = 0;
      let captureError: Error | undefined;
      let cancelled = token?.isCancellationRequested ?? false;
      const cancellation = token?.onCancellationRequested(() => {
        cancelled = true;
        child.kill();
      });
      child.stdout.setEncoding("utf8");
      child.stderr.setEncoding("utf8");
      child.stdout.on("data", (chunk: string) => {
        this.output.append(chunk);
        if (captureError !== undefined) return;
        stdoutBytes += Buffer.byteLength(chunk, "utf8");
        if (stdoutBytes > MAX_CAPTURE_BYTES) {
          captureError = new Error("FPAS stdout capture exceeds 16 MiB; command stopped. No partial report was parsed. See Functional Pascal output.");
          child.kill();
        } else stdout += chunk;
      });
      child.stderr.on("data", (chunk: string) => {
        this.output.append(chunk);
        if (captureError !== undefined) return;
        stderrBytes += Buffer.byteLength(chunk, "utf8");
        if (stderrBytes > MAX_CAPTURE_BYTES) {
          captureError = new Error("FPAS stderr capture exceeds 16 MiB; command stopped. See Functional Pascal output.");
          child.kill();
        } else stderr += chunk;
      });
      child.once("error", (error) => {
        cancellation?.dispose();
        reject(error);
      });
      child.once("close", (exitCode) => {
        cancellation?.dispose();
        if (captureError !== undefined) {
          reject(captureError);
          return;
        }
        resolve({ exitCode, stdout, stderr, cancelled });
      });
      if (cancelled) {
        child.kill();
      }
    });
  }
}
