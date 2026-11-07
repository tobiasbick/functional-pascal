import path from 'node:path';
import * as vscode from 'vscode';
import { ClientSession } from './client';
import { ContextSelection, requireTrusted, settings } from './context';
import { ToolchainResolver } from './toolchain';
import { SourceViews } from './source-views';
import { applyCorrection } from './corrections';
import { Refactorings } from './refactorings';
import { Projects } from './workflows/projects';
import { EditorTests } from './workflows/testing';
import { registerTasks } from './workflows/tasks';
import { registerDebug } from './workflows/debug';

let controller: ExtensionController | undefined;

export function activate(context: vscode.ExtensionContext): void {
  controller = new ExtensionController(context);
  void controller.restart(false);
}

export async function deactivate(): Promise<void> {
  await controller?.stop();
  controller = undefined;
}

class ExtensionController {
  private readonly contexts: ContextSelection;
  private readonly toolchain = new ToolchainResolver();
  private readonly output = vscode.window.createOutputChannel('Functional Pascal CLR', { log: true });
  private readonly status = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left);
  private client?: ClientSession;
  private readonly views = new SourceViews(() => this.client);
  private readonly refactorings = new Refactorings(() => this.client);
  private readonly projects: Projects;
  private readonly tests: EditorTests;
  private queue: Promise<void> = Promise.resolve();
  private revision = 0;
  private stopped = false;

  constructor(context: vscode.ExtensionContext) {
    this.contexts = new ContextSelection(context.workspaceState);
    this.projects = new Projects(() => this.contexts.current(), this.toolchain, this.output);
    this.tests = new EditorTests(this.projects);
    this.projects.register(context);
    this.tests.register(context);
    registerTasks(context, this.projects);
    registerDebug(context, this.projects);
    this.views.register(context);
    this.refactorings.register(context);
    const command = (name: string, action: (...args: never[]) => unknown) =>
      vscode.commands.registerCommand(`${settings}.${name}`, action);
    context.subscriptions.push(this.output, this.status,
      command('selectContext', async (uri?: vscode.Uri) => this.action(async () => {
        if (await this.contexts.select(uri)) await this.restart(false);
      })),
      command('selectExecutable', () => this.action(() => this.selectExecutable())),
      command('selectConfiguration', () => this.action(() => this.selectConfiguration())),
      command('restartLanguageServer', () => this.restart(true)),
      command('showOutput', () => this.output.show()),
      command('applyCorrection', async (request: object) => {
        try { return await applyCorrection(request, () => this.client); }
        catch (error) {
          if (!(error instanceof vscode.CancellationError))
            void vscode.window.showWarningMessage(error instanceof Error ? error.message : String(error));
          return false;
        }
      }),
      vscode.workspace.onDidGrantWorkspaceTrust(() => void this.restart(false)),
      vscode.workspace.onDidChangeWorkspaceFolders(() => void this.restart(false)),
      vscode.workspace.onDidChangeConfiguration(event => {
        if (event.affectsConfiguration(`${settings}.executablePath`) || event.affectsConfiguration(`${settings}.configuration`)) {
          void this.restart(false);
        }
      }));
    this.status.show();
  }

  restart(refresh: boolean): Promise<void> {
    const revision = ++this.revision;
    this.views.invalidate();
    this.refactorings.invalidate();
    this.projects.changed();
    this.tests.invalidate(true);
    return this.enqueue(async () => {
      if (this.stopped || revision !== this.revision) return;
      try {
        await this.close();
        if (!vscode.workspace.isTrusted) {
          this.showStatus('FPAS: Restricted Mode', 'Trust the workspace to enable FPAS project analysis.', 'showOutput');
          return;
        }
        const context = await this.contexts.current();
        if (!context) {
          this.showStatus('FPAS: Select context', 'Select a .fpasproj or .slnx for analysis.', 'selectContext');
          return;
        }
        this.showStatus('FPAS: Starting', context, 'showOutput');
        const configuration = vscode.workspace.getConfiguration(settings).get<string>('configuration', 'Debug').trim();
        if (!configuration) throw new Error('Choose a nonempty build configuration.');
        const { executable, report } = await this.toolchain.resolve(context, refresh);
        if (this.stopped || revision !== this.revision) return;
        this.output.info(`Using ${executable} (${report.fpas.cli.version}) with .NET SDK ${report.dotnet.selectedSdk}.`);
        this.output.info(`FPAS SDK cache ${report.sdkCache.status}. For setup errors, restore ${context} with the matching SDK and rebuild changed foreign references. For linked source, select its owning .fpasproj.`);
        const client = new ClientSession(executable, context, configuration, this.output, () => {
          if (revision === this.revision) this.failed('The FPAS language server exited. Restart it after checking its output.');
          void this.enqueue(async () => { if (this.client === client) await this.close(); });
        }, () => this.views.invalidate(), (uri, diagnostics) => {
          if (this.client === client) this.projects.problems.onLive(uri, diagnostics);
        });
        this.client = client;
        await client.start();
        if (this.stopped || revision !== this.revision) { await this.close(); return; }
        this.showStatus(`FPAS: ${path.basename(context)} (${configuration})`, context, 'selectContext');
      } catch (error) {
        if (!this.stopped && revision === this.revision) {
          this.failed(`${error instanceof Error ? error.message : String(error)} Select a compatible compiler, restore the project, or rebuild changed dependencies.`);
        }
      }
    });
  }

  async stop(): Promise<void> {
    this.stopped = true;
    this.revision++;
    await this.projects.stop();
    await this.enqueue(() => this.close());
  }

  private async close(): Promise<void> {
    const client = this.client;
    this.client = undefined;
    this.views.invalidate();
    this.refactorings.invalidate();
    await client?.stop();
  }

  private enqueue(action: () => Promise<void>): Promise<void> {
    const pending = this.queue.then(action);
    this.queue = pending.catch(error => this.output.error(String(error)));
    return pending;
  }

  private showStatus(text: string, tooltip: string, command: string): void {
    this.status.text = text;
    this.status.tooltip = tooltip;
    this.status.command = `${settings}.${command}`;
  }

  private failed(message: string): void {
    this.output.error(message);
    this.showStatus('$(error) FPAS: Setup required', message, 'showOutput');
  }

  private async action(action: () => Promise<void>): Promise<void> {
    try { requireTrusted(); await action(); }
    catch (error) { this.failed(error instanceof Error ? error.message : String(error)); }
  }

  private async selectExecutable(): Promise<void> {
    const selection = await vscode.window.showOpenDialog({ canSelectMany: false, openLabel: 'Select FPAS Executable' });
    if (selection?.[0]?.scheme === 'file') {
      await vscode.workspace.getConfiguration(settings).update('executablePath', selection[0].fsPath, vscode.ConfigurationTarget.Global);
    }
  }

  private async selectConfiguration(): Promise<void> {
    const configuration = vscode.workspace.getConfiguration(settings);
    const selected = await vscode.window.showInputBox({
      prompt: 'FPAS build configuration', value: configuration.get('configuration', 'Debug'),
      validateInput: value => value.trim() ? undefined : 'Enter a build configuration.',
    });
    if (selected) await configuration.update('configuration', selected.trim(), vscode.ConfigurationTarget.Workspace);
  }
}
