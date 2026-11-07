import { spawn } from 'node:child_process';
import * as vscode from 'vscode';
import { requireTrusted } from '../context';

export interface ProcessResult<T> { code: number; report: T; stderr: string }
interface Invocation { cancel(): void; done: Promise<unknown> }

export class WorkflowProcesses {
  private readonly active = new Set<Invocation>();

  async run<T>(executable: string, args: string[], cwd: string, token?: vscode.CancellationToken): Promise<ProcessResult<T>> {
    requireTrusted();
    if (token?.isCancellationRequested) throw new vscode.CancellationError();
    const child = spawn(executable, ['--editor-control', ...args], { cwd, windowsHide: true });
    const stdout: Buffer[] = [], stderr: Buffer[] = [];
    let bytes = 0;
    let overflow = false;
    let grace: NodeJS.Timeout | undefined;
    let cancelled = false;
    const cancel = () => {
      if (cancelled) return;
      cancelled = true;
      child.stdin.end();
      grace = setTimeout(() => child.kill(), 15_000);
    };
    child.stdin.on('error', () => { /* The operation may have already exited. */ });
    const read = (chunks: Buffer[]) => (chunk: Buffer) => {
      bytes += chunk.length;
      if (bytes > 6 * 1024 * 1024) { overflow = true; cancel(); }
      else chunks.push(chunk);
    };
    child.stdout.on('data', read(stdout));
    child.stderr.on('data', read(stderr));
    const done = new Promise<number>((resolve, reject) => {
      child.once('error', reject);
      child.once('close', (code, signal) => {
        if (code === null) reject(new Error(`FPAS exited with signal ${signal} before acknowledging completion.`));
        else resolve(code);
      });
    });
    const invocation = { cancel, done };
    this.active.add(invocation);
    const subscription = token?.onCancellationRequested(cancel);
    if (token?.isCancellationRequested) cancel();
    try {
      const code = await done;
      if (overflow) throw new Error('FPAS exceeded the editor output limit.');
      const output = Buffer.concat(stdout).toString('utf8');
      const error = Buffer.concat(stderr).toString('utf8');
      try { return { code, report: JSON.parse(output) as T, stderr: error }; }
      catch { throw new Error(`FPAS returned an invalid report (exit ${code}). ${error.slice(0, 4096)}`); }
    } finally {
      subscription?.dispose();
      clearTimeout(grace);
      this.active.delete(invocation);
    }
  }

  async stop(): Promise<void> {
    const running = [...this.active];
    for (const invocation of running) invocation.cancel();
    await Promise.allSettled(running.map(invocation => invocation.done));
  }
}
