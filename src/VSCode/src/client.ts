import { spawn, type ChildProcessWithoutNullStreams } from 'node:child_process';
import { once } from 'node:events';
import path from 'node:path';
import * as vscode from 'vscode';
import { CloseAction, ErrorAction, LanguageClient, RevealOutputChannelOn } from 'vscode-languageclient/node';
import { requireTrusted } from './context';

function deadline<T>(work: Promise<T>, milliseconds: number, message: string): Promise<T> {
  let timeout: NodeJS.Timeout;
  const expired = new Promise<never>((_, reject) => {
    timeout = setTimeout(() => reject(new Error(message)), milliseconds);
  });
  return Promise.race([work, expired]).finally(() => clearTimeout(timeout));
}

export class ClientSession {
  private readonly client: LanguageClient;
  private child?: ChildProcessWithoutNullStreams;
  private exited?: Promise<void>;
  private closing = false;

  constructor(executable: string, context: string, configuration: string,
    private readonly output: vscode.LogOutputChannel, onUnexpectedClose: () => void, onViewsInvalidated: () => void,
    onDiagnostics: (uri: vscode.Uri, diagnostics: vscode.Diagnostic[]) => void) {
    this.client = new LanguageClient('functionalPascalClr', 'Functional Pascal CLR', async () => {
      requireTrusted();
      const child = spawn(executable, ['lsp'], { cwd: path.dirname(context), windowsHide: true });
      this.child = child;
      this.exited = new Promise(resolve => child.once('close', () => resolve()));
      await once(child, 'spawn');
      output.info(`LSP process ${child.pid} started for ${context} (${configuration}).`);
      return child;
    }, {
      documentSelector: [{ scheme: 'file', language: 'fpas' }],
      initializationOptions: { context, configuration, correctionCommand: true },
      outputChannel: output,
      revealOutputChannelOn: RevealOutputChannelOn.Never,
      markdown: { isTrusted: false, supportHtml: false },
      middleware: { handleDiagnostics: (uri, diagnostics, next) => { onDiagnostics(uri, diagnostics); next(uri, diagnostics); } },
      initializationFailedHandler: () => false,
      errorHandler: {
        error: error => {
          output.error(`LSP transport error: ${error.message}`);
          return { action: ErrorAction.Shutdown, handled: true };
        },
        closed: () => {
          if (!this.closing) onUnexpectedClose();
          return { action: CloseAction.DoNotRestart, handled: true };
        },
      },
    });
    this.client.onNotification('fpas/viewsInvalidated', onViewsInvalidated);
  }

  request<T>(method: string, parameters: object, token: vscode.CancellationToken): Promise<T> {
    requireTrusted();
    return this.client.sendRequest<T>(method, parameters, token);
  }

  async start(): Promise<void> {
    try {
      await deadline(this.client.start(), 20_000, 'FPAS language server initialization timed out.');
    } catch (error) {
      await this.stop();
      throw error;
    }
  }

  async stop(): Promise<void> {
    if (this.closing) return;
    this.closing = true;
    try {
      await deadline(this.client.dispose(4_000), 5_000, 'LSP shutdown timed out.');
    } catch (error) {
      this.output.warn(String(error));
    }
    if (!this.child || !this.exited) return;

    // EOF lets the CLR server cancel and join its owned MSBuild processes.
    this.child.stdin.end();
    try {
      await deadline(this.exited, 3_000, 'LSP process did not exit after shutdown.');
    } catch (error) {
      this.output.warn(String(error));
      this.child.kill('SIGKILL');
      await deadline(this.exited, 3_000, 'Could not reap the FPAS language server.');
    }
    this.output.info(`LSP process ${this.child.pid} exited.`);
  }
}
