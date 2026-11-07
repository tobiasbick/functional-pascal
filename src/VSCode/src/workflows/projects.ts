import path from 'node:path';
import { stat } from 'node:fs/promises';
import * as vscode from 'vscode';
import { requireTrusted, settings } from '../context';
import { ToolchainResolver } from '../toolchain';
import { CommandProblems, type CapturedBuild, type ReportDiagnostic } from './problems';
import { WorkflowProcesses } from './process';

export const operations = ['check', 'restore', 'build', 'clean', 'run', 'test', 'format'] as const;
export type Operation = typeof operations[number];
export interface WorkflowOptions {
  configuration?: string; project?: string; args?: string[]; cwd?: string; noBuild?: boolean; captured?: boolean;
}
export interface Selection { context: string; configuration: string; executable: string; cwd: string; epoch: number }
interface Target { project: string; configuration: string; assembly: string }
interface WorkflowReport {
  success: boolean; status?: string; target?: Target; error?: { code: string; message: string; candidates?: string[] };
  build?: CapturedBuild; diagnostics?: ReportDiagnostic[]; preparation?: { issues: { message: string }[] };
  program?: { output: string; error: string; exitCode: number };
  projects?: { runner?: CapturedBuild; resultsFile?: string }[];
}

export class Projects {
  private epoch = 0;
  private readonly status = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left);
  private readonly runTasks = new Set<vscode.TaskExecution>();
  private runProject?: string;
  private testProject?: string;
  readonly processes = new WorkflowProcesses();
  readonly problems = new CommandProblems();

  constructor(private readonly current: () => Promise<string | undefined>,
    readonly toolchain: ToolchainResolver, readonly output: vscode.LogOutputChannel) {
    this.status.command = settings + '.selectRunProject';
  }

  register(context: vscode.ExtensionContext): void {
    context.subscriptions.push(this.status, this.problems, vscode.tasks.onDidEndTask(event => this.runTasks.delete(event.execution)),
      vscode.commands.registerCommand(settings + '.selectRunProject', () => this.guard(async () => {
        const selection = await this.selection();
        await this.target(selection, undefined, undefined, true);
      })));
    for (const operation of operations)
      context.subscriptions.push(vscode.commands.registerCommand(settings + '.' + operation, (options?: WorkflowOptions) => this.guard(() =>
        vscode.window.withProgress({ location: vscode.ProgressLocation.Notification, title: 'FPAS ' + operation, cancellable: true },
          async (_, token) => {
            if (operation === 'run' && options?.captured !== true) {
              const task = await this.interactive(options ?? {}, token);
              const execution = await vscode.tasks.executeTask(task);
              this.runTasks.add(execution);
              return execution;
            }
            return this.captured(operation, options ?? {}, token);
          }))));
  }

  changed(): void {
    this.epoch++;
    this.runProject = undefined;
    this.testProject = undefined;
    this.problems.invalidate(true);
    this.status.hide();
  }

  private show(selection: Selection): void {
    this.status.text = `FPAS Run ${this.runProject ? path.basename(this.runProject) : 'Auto'} · Tests ${this.testProject ? path.basename(this.testProject) : 'All'} · ${selection.configuration}`;
    this.status.tooltip = [selection.context, this.runProject ?? 'Automatic executable selection', this.testProject ?? 'All included test projects'].join('\n');
    this.status.show();
  }

  selectTests(selection: Selection, project?: string): void {
    this.testProject = project;
    this.show(selection);
  }

  async stop(): Promise<void> {
    for (const execution of this.runTasks) execution.terminate();
    await this.processes.stop();
  }

  async selection(options: WorkflowOptions = {}): Promise<Selection> {
    requireTrusted();
    const epoch = this.epoch;
    const context = await this.current();
    if (!context) throw new Error('Select an FPAS analysis context first.');
    const configuration = options.configuration ?? vscode.workspace.getConfiguration(settings).get('configuration', 'Debug');
    if (typeof configuration !== 'string' || !configuration.trim() || configuration.includes('\0')) throw new Error('Choose a valid build configuration.');
    const { executable } = await this.toolchain.resolve(context, false);
    const selected = { context, configuration, executable, cwd: path.dirname(context), epoch };
    this.verify(selected);
    this.show(selected);
    return selected;
  }

  verify(selection: Selection): void {
    requireTrusted();
    if (selection.epoch !== this.epoch) throw new vscode.CancellationError();
  }

  async diskConsent(selection: Selection): Promise<void> {
    this.verify(selection);
    const dirty = vscode.workspace.textDocuments.filter(document => document.isDirty && document.uri.scheme === 'file' &&
      /\.(?:fpas|cs|fs|fsi|fpasproj|csproj|fsproj|slnx|props|targets|json|config)$/i.test(document.uri.fsPath));
    if (dirty.length) {
      const choice = await vscode.window.showWarningMessage('This FPAS operation uses files on disk. Save the open changes first?',
        { modal: true }, 'Save and Continue', 'Use Files on Disk');
      if (!choice) throw new vscode.CancellationError();
      if (choice === 'Save and Continue') for (const document of dirty)
        if (!await document.save()) throw new Error(`Could not save ${document.fileName}.`);
    }
    this.verify(selection);
  }

  async target(selection: Selection, project?: string, token?: vscode.CancellationToken, choose = false): Promise<Target> {
    this.verify(selection);
    const resolve = (selected?: string) => this.processes.run<WorkflowReport>(selection.executable,
      ['target', selection.context, '--configuration', selection.configuration, ...projectArguments(selected)], selection.cwd, token);
    const report = (await resolve(project ?? (choose ? undefined : this.runProject))).report;
    this.verify(selection);
    if (report.target) {
      if (choose || !project) { this.runProject = report.target.project; this.show(selection); }
      return report.target;
    }
    if (!project && report.error?.code === 'ambiguous_project' && report.error.candidates?.length) {
      const picked = await vscode.window.showQuickPick(report.error.candidates.map(candidate =>
        ({ label: path.basename(candidate), description: vscode.workspace.asRelativePath(candidate), project: candidate })),
      { placeHolder: 'Select an included executable project' });
      if (!picked) throw new vscode.CancellationError();
      const chosen = (await resolve(picked.project)).report;
      this.verify(selection);
      if (chosen.target) { this.runProject = chosen.target.project; this.show(selection); return chosen.target; }
      throw new Error(chosen.error?.message ?? 'FPAS did not return a target.');
    }
    throw new Error(report.error?.message ?? 'FPAS did not return a target.');
  }

  async interactive(options: WorkflowOptions, token?: vscode.CancellationToken): Promise<vscode.Task> {
    const selection = await this.selection(options);
    await this.diskConsent(selection);
    const target = await this.target(selection, options.project, token);
    const cwd = await workingDirectory(options.cwd, path.dirname(target.project));
    this.verify(selection);
    // VS Code resolves task variables before starting the process. Encode the already selected argv once.
    const payload = Buffer.from(JSON.stringify(this.runArguments(selection, target, options, cwd, false)), 'utf8').toString('base64');
    if (payload.length > 24_000) throw new Error('FPAS terminal arguments exceed the supported length.');
    const execution = new vscode.ProcessExecution(selection.executable, ['--editor-run', payload], { cwd: selection.cwd });
    const task = new vscode.Task({ type: 'fpas', operation: 'run' }, this.scope(selection), `Run ${path.basename(target.project)}`, 'FPAS', execution);
    task.presentationOptions = { reveal: vscode.TaskRevealKind.Always, focus: true, panel: vscode.TaskPanelKind.Dedicated };
    return task;
  }

  scope(selection: Selection): vscode.WorkspaceFolder | vscode.TaskScope {
    return vscode.workspace.getWorkspaceFolder(vscode.Uri.file(selection.context)) ?? vscode.TaskScope.Workspace;
  }

  async captured(operation: Operation, options: WorkflowOptions, token: vscode.CancellationToken, write?: (text: string) => void): Promise<number> {
    const selection = await this.selection(options);
    await this.diskConsent(selection);
    if (token.isCancellationRequested) throw new vscode.CancellationError();
    let args = [operation === 'format' ? 'fmt' : operation, selection.context, '--configuration', selection.configuration,
      operation === 'check' ? '--diagnostics' : '--report', 'json'];
    if (operation === 'run') {
      const target = await this.target(selection, options.project, token);
      const cwd = await workingDirectory(options.cwd, path.dirname(target.project));
      args = this.runArguments(selection, target, options, cwd, true);
    } else if (operation === 'test') args.push(...projectArguments(options.project ?? this.testProject), ...(options.noBuild ? ['--no-build'] : []));
    const stamp = this.problems.begin();
    const result = await this.processes.run<WorkflowReport>(selection.executable, args, selection.cwd, token);
    this.verify(selection);
    const report = result.report;
    const diagnostics = report.diagnostics ?? report.build?.diagnostics ?? [];
    if (!token.isCancellationRequested) this.problems.publish(stamp, operation, diagnostics, selection.cwd);
    const content = [report.build?.output, report.build?.error, ...diagnostics.map(item => `${item.severity} ${item.code ?? ''} ${item.message}`),
      ...report.preparation?.issues.map(issue => issue.message) ?? [], ...report.projects?.flatMap(item =>
        [item.runner?.output, item.runner?.error, item.resultsFile]) ?? [], report.program?.output, report.program?.error, result.stderr,
      report.error?.message, report.build?.diagnosticsTruncated ? 'MSBuild diagnostic report was truncated.' : '', `FPAS ${operation} exited ${result.code}.`]
      .filter(Boolean).join('\n');
    this.output.appendLine(content);
    write?.(content);
    if (result.code && result.code !== 130 && !write) this.output.show(true);
    return result.code;
  }

  async guard<T>(work: () => PromiseLike<T>): Promise<T | undefined> {
    try { requireTrusted(); return await work(); }
    catch (error) {
      if (!(error instanceof vscode.CancellationError)) {
        const message = error instanceof Error ? error.message : String(error);
        this.output.error(message);
        void vscode.window.showWarningMessage(message);
      }
      return undefined;
    }
  }

  private runArguments(selection: Selection, target: Target, options: WorkflowOptions, cwd: string, captured: boolean): string[] {
    const args = options.args ?? vscode.workspace.getConfiguration(settings).get('run.arguments', []);
    if (!Array.isArray(args) || args.some(item => typeof item !== 'string' || item.includes('\0'))) throw new Error('Run arguments must be an array of strings.');
    return ['run', selection.context, '--configuration', selection.configuration, ...projectArguments(target.project),
      '--working-directory', cwd, ...(options.noBuild ? ['--no-build'] : []), ...(captured ? ['--report', 'json', '--timeout', '120'] : []), '--', ...args];
  }
}

export function projectArguments(project?: string): string[] {
  if (project === undefined) return [];
  if (typeof project !== 'string' || !path.isAbsolute(project) || project.includes('\0')) throw new Error('Select an absolute project path on this host.');
  return ['--project', project];
}

export async function workingDirectory(selected: string | undefined, fallback: string): Promise<string> {
  const directory = selected ?? (vscode.workspace.getConfiguration(settings).get<string>('run.workingDirectory')?.trim() || fallback);
  if (typeof directory !== 'string' || !path.isAbsolute(directory) || !(await stat(directory)).isDirectory())
    throw new Error('Select an existing absolute working directory on this host.');
  return directory;
}
