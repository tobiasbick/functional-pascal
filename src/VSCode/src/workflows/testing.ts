import path from 'node:path';
import * as vscode from 'vscode';
import { settings } from '../context';
import { Projects, projectArguments, type Selection } from './projects';
import { range, type CapturedBuild, type SourceRange } from './problems';

interface NativeProject {
  project: string; configuration: string; platform: string; assembly: string; testingPlatform: boolean;
  outputIdentity: string; staleSources: boolean;
}
interface Discovered { fullyQualifiedName: string; displayName?: string; uid?: string; location?: SourceRange }
interface Case {
  id?: string; fullyQualifiedName?: string; name: string; outcome: string; duration?: string;
  output?: string; errorOutput?: string; error?: string; stackTrace?: string;
}
interface NativeReport {
  project: NativeProject; runner: CapturedBuild; resultsFile?: string; discovery?: Discovered[];
  result?: { tests: Case[]; messages: { outcome: string; text: string }[]; output?: string; errorOutput?: string };
}
interface Report {
  success: boolean; status: string; error?: { code: string; message: string };
  resultsDirectory?: string; projects: NativeReport[];
}
interface Group { project: NativeProject; root: vscode.TestItem; tests: Map<string, Discovered> }

export class EditorTests implements vscode.Disposable {
  private readonly controller = vscode.tests.createTestController('functionalPascalClr', 'Functional Pascal');
  private readonly watchers: vscode.Disposable[] = [];
  private groups = new Map<string, Group>();
  private generation = 0;
  private discovery?: { done: Promise<void>; cancel: vscode.CancellationTokenSource };
  private selected?: string;

  constructor(private readonly projects: Projects) {
    const controller = this.controller;
    controller.resolveHandler = async () => { if (!this.groups.size) await this.refresh(false); };
    controller.refreshHandler = token => this.refresh(true, token);
    controller.createRunProfile('Run FPAS Tests', vscode.TestRunProfileKind.Run, (request, token) => this.run(request, token), true);
    const watcher = vscode.workspace.createFileSystemWatcher('**/*.{dll,pdb,fpas,cs,fs,fpasproj,csproj,fsproj,slnx,props,targets}');
    const changed = (uri: vscode.Uri) => {
      if (/\.(dll|pdb)$/i.test(uri.fsPath)) {
        const output = uri.fsPath.replace(/\.(dll|pdb)$/i, '.dll');
        if (![...this.groups.values()].some(group => samePath(group.project.assembly, output))) return;
      } else if (uri.fsPath.split(path.sep).some(part => ['bin', 'obj', 'node_modules', 'TestResults'].includes(part))) return;
      this.invalidate();
    };
    this.watchers.push(watcher, watcher.onDidChange(changed), watcher.onDidDelete(changed), watcher.onDidCreate(changed));
  }

  register(context: vscode.ExtensionContext): void {
    context.subscriptions.push(this,
      vscode.commands.registerCommand(settings + '.refreshTests', () => this.projects.guard(() =>
        vscode.window.withProgress({ location: vscode.ProgressLocation.Notification, title: 'Discover FPAS tests', cancellable: true },
          (_, token) => this.refresh(true, token)))),
      vscode.commands.registerCommand(settings + '.buildTests', () => this.projects.guard(() =>
        vscode.window.withProgress({ location: vscode.ProgressLocation.Notification, title: 'Build FPAS test outputs', cancellable: true },
          async (_, token) => {
            if (await this.projects.captured('build', {}, token) === 0) await this.refresh(true, token);
          }))),
      vscode.commands.registerCommand(settings + '.selectTestProject', () => this.projects.guard(async () => {
        const selection = await this.projects.selection();
        await this.refresh(true);
        const choice = await vscode.window.showQuickPick([
          { label: 'All included test projects', project: undefined },
          ...[...this.groups.values()].map(group => ({ label: path.basename(group.project.project),
            description: group.project.project, project: group.project.project })),
        ], { placeHolder: 'Select test project' });
        if (choice) { this.selected = choice.project; this.projects.selectTests(selection, choice.project); this.showGroups(); }
      })));
  }

  invalidate(contextChanged = false): void {
    this.generation++;
    this.discovery?.cancel.cancel();
    this.discovery = undefined;
    this.groups.clear();
    this.controller.items.replace([]);
    if (contextChanged) this.selected = undefined;
  }

  private refresh(explicit: boolean, token?: vscode.CancellationToken): Promise<void> {
    if (!vscode.workspace.isTrusted) return Promise.resolve();
    if (this.discovery) return this.discovery.done;
    const cancel = new vscode.CancellationTokenSource();
    const subscription = token?.onCancellationRequested(() => cancel.cancel());
    if (token?.isCancellationRequested) cancel.cancel();
    const done = this.discover(explicit, cancel.token).finally(() => {
      if (this.discovery?.cancel === cancel) this.discovery = undefined;
      subscription?.dispose();
      cancel.dispose();
    });
    this.discovery = { done, cancel };
    return done;
  }

  private async discover(explicit: boolean, token?: vscode.CancellationToken): Promise<void> {
    const generation = ++this.generation;
    try {
      const selection = await this.projects.selection();
      const { report } = await this.request(selection, ['--list'], token);
      this.projects.verify(selection);
      if (generation !== this.generation || token?.isCancellationRequested) return;
      const groups = new Map<string, Group>();
      for (const native of report.projects) {
        if (!native.discovery || native.runner.error && !native.project.outputIdentity) continue;
        const project = native.project;
        const id = JSON.stringify([project.project, project.configuration, project.platform]);
        const root = this.controller.createTestItem(id, `${path.basename(project.project)} (${project.configuration})`, vscode.Uri.file(project.project));
        const tests = new Map<string, Discovered>();
        if (project.staleSources) root.error = 'Test sources differ from the built output. Run Functional Pascal: Build Test Outputs.';
        for (const discovered of native.discovery) {
          if (project.testingPlatform && !discovered.uid) continue;
          const key = JSON.stringify([id, discovered.uid ?? discovered.fullyQualifiedName]);
          const item = this.controller.createTestItem(key, discovered.displayName || discovered.fullyQualifiedName,
            discovered.location ? vscode.Uri.file(discovered.location.path) : undefined);
          if (discovered.location) item.range = range(discovered.location);
          root.children.add(item);
          tests.set(key, discovered);
        }
        groups.set(id, { project, root, tests });
      }
      this.groups = groups;
      this.showGroups();
      if (!report.success) {
        const error = report.error?.message ?? 'FPAS test discovery failed.';
        this.projects.output.error(error);
        if (explicit && report.error?.code === 'missing_test_output') {
          void vscode.window.showWarningMessage(error, 'Build Test Outputs').then(choice => {
            if (choice) void vscode.commands.executeCommand(settings + '.buildTests');
          });
        } else if (explicit) void vscode.window.showWarningMessage(error);
      }
    } catch (error) {
      if (!(error instanceof vscode.CancellationError) && explicit)
        this.projects.output.error(String(error));
    }
  }

  private showGroups(): void {
    this.controller.items.replace([...this.groups.values()].filter(group => !this.selected || samePath(group.project.project, this.selected))
      .map(group => group.root));
  }

  private async run(request: vscode.TestRunRequest, token: vscode.CancellationToken): Promise<void> {
    if (!vscode.workspace.isTrusted) return;
    if (!this.groups.size && !request.include) await this.refresh(true, token);
    const run = this.controller.createTestRun(request);
    const includes = new Set(request.include?.map(item => item.id));
    const excludes = new Set(request.exclude?.map(item => item.id));
    try {
      const selection = await this.projects.selection();
      await this.projects.diskConsent(selection);
      const groups = [...this.groups.values()].filter(group => !this.selected || samePath(group.project.project, this.selected));
      for (const group of groups) {
        if (excludes.has(group.root.id)) continue;
        const all = !request.include || includes.has(group.root.id);
        const items = [...group.tests].filter(([key]) => (all || includes.has(key)) && !excludes.has(key));
        if (!items.length) continue;
        if (group.project.staleSources) {
          for (const [key] of items) run.errored(group.root.children.get(key)!, new vscode.TestMessage('Build and refresh stale test output first.'));
          continue;
        }
        if (all && items.length === group.tests.size)
          await this.execute(selection, group, items, run, token, false);
        else for (const item of items) await this.execute(selection, group, [item], run, token, true);
      }
      for (const item of request.include ?? [])
        if (!groups.some(group => group.root.id === item.id || group.tests.has(item.id)))
          run.errored(item, new vscode.TestMessage('This test selection is stale. Refresh the test view.'));
    } catch (error) {
      if (!(error instanceof vscode.CancellationError)) run.appendOutput(outputLine(String(error)));
    } finally { run.end(); }
  }

  private async execute(selection: Selection, group: Group, entries: [string, Discovered][], run: vscode.TestRun,
    token: vscode.CancellationToken, single: boolean): Promise<void> {
    const items = entries.map(([key]) => group.root.children.get(key)!);
    if (token.isCancellationRequested) { for (const item of items) run.skipped(item); return; }
    for (const item of items) run.started(item);
    const test = entries[0][1];
    const filter = !single ? [] : group.project.testingPlatform ? ['--', '--filter-uid', test.uid!] :
      ['--filter', 'FullyQualifiedName=' + test.fullyQualifiedName.replace(/[\\()&|=!~]/g, '\\$&')];
    try {
      const { report } = await this.request(selection, [...projectArguments(group.project.project),
        '--expected-output', group.project.outputIdentity, ...filter], token);
      this.projects.verify(selection);
      const native = report.projects.find(entry => samePath(entry.project.project, group.project.project));
      for (const value of [native?.runner.output, native?.runner.error, native?.result?.output, native?.result?.errorOutput,
        ...native?.result?.messages.map(message => message.text) ?? [], native?.resultsFile ? `TRX ${vscode.Uri.file(native.resultsFile)}` : report.resultsDirectory,
        report.error?.message])
        if (value) run.appendOutput(outputLine(value));
      if (report.status === 'cancelled' || token.isCancellationRequested) { for (const item of items) run.skipped(item); return; }
      if (!native?.result || report.error) {
        const message = new vscode.TestMessage(report.error?.message ?? 'The runner did not return a test result.');
        for (const item of items) run.errored(item, message);
        return;
      }
      const mapped = matchCases(group, entries, native.result.tests, single);
      const correlated = [...mapped.values()].reduce((count, cases) => count + cases.length, 0);
      if (correlated !== native.result.tests.length) {
        for (const item of items) run.errored(item, new vscode.TestMessage('Incomplete result. The runner returned cases outside the verified selection.'));
        return;
      }
      for (let index = 0; index < entries.length; index++) {
        const [key] = entries[index];
        const item = items[index];
        const cases = mapped.get(key);
        if (!cases?.length) { run.errored(item, new vscode.TestMessage('Incomplete result. No native case could be correlated with this test identity.')); continue; }
        publish(run, item, cases);
      }
    } catch (error) {
      for (const item of items)
        if (error instanceof vscode.CancellationError || token.isCancellationRequested) run.skipped(item);
        else run.errored(item, new vscode.TestMessage(error instanceof Error ? error.message : String(error)));
    }
  }

  private request(selection: Selection, extra: string[], token?: vscode.CancellationToken) {
    this.projects.verify(selection);
    return this.projects.processes.run<Report>(selection.executable,
      ['test', selection.context, '--configuration', selection.configuration, '--no-build', '--report', 'json', ...extra], selection.cwd, token);
  }

  dispose(): void {
    this.discovery?.cancel.cancel();
    for (const watcher of this.watchers) watcher.dispose();
    this.controller.dispose();
  }
}

function matchCases(group: Group, entries: [string, Discovered][], cases: Case[], single: boolean): Map<string, Case[]> {
  const selected = new Map(entries);
  const names = new Map<string, string[]>();
  const identities = new Map<string, string>();
  for (const [key, test] of group.tests) {
    if (test.uid) identities.set(test.uid, key);
    const matches = names.get(test.fullyQualifiedName) ?? [];
    matches.push(key);
    names.set(test.fullyQualifiedName, matches);
  }
  const result = new Map<string, Case[]>();
  for (const item of cases) {
    const namesakes = item.fullyQualifiedName ? names.get(item.fullyQualifiedName) ?? [] : [];
    const key = (item.id ? identities.get(item.id) : undefined) ??
      (namesakes.length === 1 ? namesakes[0] :
        single && cases.length === 1 && namesakes.includes(entries[0][0]) ? entries[0][0] : undefined);
    if (!key || !selected.has(key)) continue;
    const group = result.get(key) ?? [];
    group.push(item);
    result.set(key, group);
  }
  return result;
}

function publish(run: vscode.TestRun, item: vscode.TestItem, cases: Case[]): void {
  for (const result of cases)
    for (const value of [result.output, result.errorOutput]) if (value) run.appendOutput(outputLine(value), undefined, item);
  const duration = cases.reduce((total, value) => total + milliseconds(value.duration), 0);
  const failed = cases.filter(value => value.outcome === 'Failed').map(value =>
    new vscode.TestMessage([value.name, value.error, value.stackTrace].filter(Boolean).join('\n')));
  if (failed.length) run.failed(item, failed, duration);
  else if (cases.some(value => !['Passed', 'NotExecuted'].includes(value.outcome)))
    run.errored(item, new vscode.TestMessage('Native result: ' + cases.map(value => value.outcome).join(', ')), duration);
  else if (cases.every(value => value.outcome === 'NotExecuted')) run.skipped(item);
  else run.passed(item, duration);
}

function milliseconds(value?: string): number {
  const match = /^(?:(\d+)\.)?(\d+):(\d+):(\d+(?:\.\d+)?)$/.exec(value ?? '');
  return match ? (((Number(match[1] ?? 0) * 24 + Number(match[2])) * 60 + Number(match[3])) * 60 + Number(match[4])) * 1000 : 0;
}

function outputLine(value: string): string { return value.replace(/\r?\n/g, '\r\n') + '\r\n'; }
function samePath(left: string, right: string): boolean { return process.platform === 'win32' ? left.toLowerCase() === right.toLowerCase() : left === right; }
