import * as vscode from 'vscode';
import { operations, Projects, type Operation, type WorkflowOptions } from './projects';

type Definition = vscode.TaskDefinition & WorkflowOptions & { operation: Operation };

export function registerTasks(context: vscode.ExtensionContext, projects: Projects): void {
  const create = (definition: Definition, scope: vscode.WorkspaceFolder | vscode.TaskScope): vscode.Task => {
    const task = new vscode.Task(definition, scope, definition.operation, 'FPAS', new vscode.CustomExecution(async () =>
      new CapturedTerminal(projects, definition)));
    if (definition.operation === 'build') task.group = vscode.TaskGroup.Build;
    if (definition.operation === 'test') task.group = vscode.TaskGroup.Test;
    if (definition.operation === 'clean') task.group = vscode.TaskGroup.Clean;
    return task;
  };
  context.subscriptions.push(vscode.tasks.registerTaskProvider('fpas', {
    async provideTasks() {
      if (!vscode.workspace.isTrusted) return [];
      const scope = vscode.workspace.workspaceFolders?.[0] ?? vscode.TaskScope.Workspace;
      return operations.filter(operation => operation !== 'run').map(operation => create({ type: 'fpas', operation }, scope));
    },
    async resolveTask(task, token) {
      if (!vscode.workspace.isTrusted || !operations.includes(task.definition.operation)) return undefined;
      const definition = task.definition as Definition;
      if (definition.operation === 'run' && !definition.captured) {
        const resolved = await projects.guard(() => projects.interactive(definition, token));
        if (!resolved?.execution) return undefined;
        const taskWithExecution = new vscode.Task(definition, task.scope ?? vscode.TaskScope.Workspace, task.name, 'FPAS', resolved.execution);
        taskWithExecution.presentationOptions = resolved.presentationOptions;
        return taskWithExecution;
      }
      return create(definition, task.scope ?? vscode.TaskScope.Workspace);
    },
  }));
}

class CapturedTerminal implements vscode.Pseudoterminal {
  private readonly output = new vscode.EventEmitter<string>();
  private readonly ended = new vscode.EventEmitter<number>();
  private readonly cancellation = new vscode.CancellationTokenSource();
  readonly onDidWrite = this.output.event;
  readonly onDidClose = this.ended.event;

  constructor(private readonly projects: Projects, private readonly definition: Definition) {}

  open(): void { void this.run(); }
  close(): void { this.cancellation.cancel(); }
  handleInput(data: string): void { if (data.includes('\u0003')) this.close(); }

  private async run(): Promise<void> {
    let code = 1;
    try {
      code = await this.projects.captured(this.definition.operation, this.definition, this.cancellation.token,
        text => this.output.fire(text.replace(/\r?\n/g, '\r\n') + '\r\n'));
    } catch (error) {
      if (error instanceof vscode.CancellationError) code = 130;
      else this.output.fire((error instanceof Error ? error.message : String(error)) + '\r\n');
    } finally {
      this.ended.fire(code);
      this.cancellation.dispose();
      this.output.dispose();
      this.ended.dispose();
    }
  }
}
