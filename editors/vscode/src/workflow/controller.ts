import path from "node:path";

import * as vscode from "vscode";

import { resolveCliPath } from "../cliPath";
import { resolveStandardLibraryPath } from "../standardLibraryPath";
import { operationArguments, runArguments } from "./arguments";
import { CliCompatibility } from "./cliCompatibility";
import {
  parseWorkflowDiagnostics,
  publishWorkflowDiagnostics
} from "./diagnostics";
import type { WorkflowOperation, WorkflowTestStatus } from "./model";
import type { ParsedWorkflowDiagnostic } from "./model";
import { WorkflowProcessRunner } from "./processes";
import { ProjectSelector } from "./project";
import { WorkflowTesting } from "./testing";

/** Commands implemented by the project workflow. */
export const WORKFLOW_COMMANDS = {
  selectProject: "functionalPascal.selectProject",
  check: "functionalPascal.checkProject",
  build: "functionalPascal.buildProject",
  run: "functionalPascal.runProject",
  test: "functionalPascal.testProject",
  format: "functionalPascal.formatProject",
  formatCheck: "functionalPascal.checkProjectFormatting",
  cancel: "functionalPascal.cancelOperation",
  refreshTests: "functionalPascal.refreshTests"
} as const;

/** Owns project commands, Problems, status, cancellation, terminal, and tests. */
export class WorkflowController implements vscode.Disposable {
  private readonly selector: ProjectSelector;
  private readonly runner: WorkflowProcessRunner;
  private readonly diagnostics = vscode.languages.createDiagnosticCollection(
    "functional-pascal-workflow"
  );
  private readonly status = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Left,
    50
  );
  private readonly testing: WorkflowTesting;
  private readonly cli: CliCompatibility;
  private cancellation: vscode.CancellationTokenSource | undefined;
  private unavailableMessage: string | undefined;
  private lastResult:
    | {
        readonly exitCode: number | null;
        readonly stderr: string;
        readonly diagnostics: readonly ParsedWorkflowDiagnostic[];
      }
    | undefined;

  public constructor(
    private readonly context: vscode.ExtensionContext,
    private readonly output: vscode.LogOutputChannel
  ) {
    this.selector = new ProjectSelector(context.workspaceState);
    this.runner = new WorkflowProcessRunner(output);
    this.cli = new CliCompatibility(() => resolveCliPath(context), this.runner);
    this.status.command = WORKFLOW_COMMANDS.selectProject;
    this.status.tooltip = "Select the active Functional Pascal project";
    this.status.show();
    this.testing = new WorkflowTesting(
      this.selector,
      this.runner,
      () => this.cli.resolve(),
      () => resolveStandardLibraryPath(context),
      async (stderr, cwd) => {
        await publishWorkflowDiagnostics(
          this.diagnostics,
          parseWorkflowDiagnostics(stderr, cwd)
        );
      },
      (running) => {
        if (running) {
          this.status.text = "$(sync~spin) FPAS: Test";
        } else {
          void this.updateStatus();
        }
      }
    );
    this.selector.onDidChange(() => {
      void this.updateStatus();
    });
    this.registerCommands();
    context.subscriptions.push(this.selector, this.diagnostics, this.status, this);
    void this.updateStatus();
  }

  /** Returns the resolved CLI path for extension-host verification. */
  public cliPath(): string {
    return resolveCliPath(this.context);
  }

  /** Selects one manifest explicitly and refreshes Testing API items. */
  public async selectProject(uri: vscode.Uri): Promise<void> {
    await this.selector.select(uri);
    await this.testing.discover();
  }

  /** Returns discovered test source paths for extension-host verification. */
  public async discoverTests(): Promise<string[]> {
    return this.testing.discover();
  }

  /** Runs selected tests through the Testing API and returns observed statuses. */
  public async runTests(files?: readonly string[]): Promise<Record<string, WorkflowTestStatus>> {
    return this.testing.runFiles(files);
  }

  /** Returns the last command result for Extension Host regression assertions. */
  public lastOperation(): typeof this.lastResult {
    return this.lastResult;
  }

  public dispose(): void {
    this.cancellation?.cancel();
    this.cancellation?.dispose();
    this.testing.dispose();
  }

  private registerCommands(): void {
    const subscriptions = [
      vscode.commands.registerCommand(
        WORKFLOW_COMMANDS.selectProject,
        async (uri?: vscode.Uri) => {
          await this.selector.select(uri);
          await this.testing.discover();
        }
      ),
      ...([
        [WORKFLOW_COMMANDS.check, "check"],
        [WORKFLOW_COMMANDS.build, "build"],
        [WORKFLOW_COMMANDS.format, "format"],
        [WORKFLOW_COMMANDS.formatCheck, "formatCheck"]
      ] as const).map(([command, operation]) =>
        vscode.commands.registerCommand(command, async (uri?: vscode.Uri) => {
          await this.runOperation(operation, uri);
        })
      ),
      vscode.commands.registerCommand(
        WORKFLOW_COMMANDS.test,
        async (uri?: vscode.Uri) => {
          const target = await this.selector.select(uri);
          if (target === undefined) {
            return;
          }
          this.status.text = "$(sync~spin) FPAS: Test";
          try {
            await this.cli.resolve();
            await this.testing.discover();
            await this.testing.runFiles();
          } catch (error) {
            await this.reportUnavailable(error);
          } finally {
            await this.updateStatus();
          }
        }
      ),
      vscode.commands.registerCommand(
        WORKFLOW_COMMANDS.run,
        async (options?: { target?: vscode.Uri; programArguments?: string[] }) => {
          await this.runInteractive(options);
        }
      ),
      vscode.commands.registerCommand(WORKFLOW_COMMANDS.cancel, () => {
        this.cancellation?.cancel();
        this.testing.cancel();
      }),
      vscode.commands.registerCommand(WORKFLOW_COMMANDS.refreshTests, async () => {
        await this.testing.discover();
      })
    ];
    this.context.subscriptions.push(...subscriptions);
  }

  private async runOperation(
    operation: WorkflowOperation,
    explicit?: vscode.Uri
  ): Promise<void> {
    const target = await this.selector.select(explicit);
    if (target === undefined) {
      return;
    }
    let cli: string;
    let standardLibrary = "";
    try {
      cli = await this.cli.resolve();
      if (operation !== "format" && operation !== "formatCheck") {
        standardLibrary = resolveStandardLibraryPath(this.context);
      }
    } catch (error) {
      await this.reportUnavailable(error);
      return;
    }
    this.cancellation?.cancel();
    this.cancellation?.dispose();
    const cancellation = new vscode.CancellationTokenSource();
    this.cancellation = cancellation;
    const label = operationLabel(operation);
    this.status.text = `$(sync~spin) FPAS: ${label}`;
    this.status.tooltip = `${label}: ${target.fsPath}`;
    this.output.appendLine(
      `\n> fpas ${operationArguments(operation, target.fsPath, standardLibrary)
        .map(renderArgument)
        .join(" ")}`
    );
    this.output.show(true);
    const cwd = path.dirname(target.fsPath);
    try {
      const result = await this.runner.run(
        cli,
        operationArguments(operation, target.fsPath, standardLibrary),
        cwd,
        cancellation.token
      );
      const parsedDiagnostics = parseWorkflowDiagnostics(result.stderr, cwd);
      this.lastResult = {
        exitCode: result.exitCode,
        stderr: result.stderr,
        diagnostics: parsedDiagnostics
      };
      await publishWorkflowDiagnostics(
        this.diagnostics,
        parsedDiagnostics
      );
      if (result.cancelled) {
        this.output.appendLine(`${label} cancelled.`);
      } else if (result.exitCode !== 0) {
        void vscode.window.showErrorMessage(
          `${label} failed with exit code ${result.exitCode}. See Functional Pascal output and Problems.`
        );
      } else {
        this.output.appendLine(`${label} completed successfully.`);
      }
      if (operation === "test") {
        await this.testing.discover();
      }
    } catch (error) {
      await this.reportUnavailable(error);
    } finally {
      if (this.cancellation === cancellation) {
        this.cancellation = undefined;
      }
      cancellation.dispose();
      await this.updateStatus();
    }
  }

  private async runInteractive(options?: {
    target?: vscode.Uri;
    programArguments?: string[];
  }): Promise<void> {
    const target = await this.selector.select(options?.target);
    if (target === undefined) {
      return;
    }
    let programArguments = options?.programArguments;
    if (programArguments === undefined) {
      const input = await vscode.window.showInputBox({
        prompt: "Program arguments as a JSON string array",
        value: "[]",
        validateInput: validateProgramArguments
      });
      if (input === undefined) {
        return;
      }
      programArguments = parseProgramArguments(input);
    }
    try {
      const cli = await this.cli.resolve();
      const standardLibrary = resolveStandardLibraryPath(this.context);
      const terminal = vscode.window.createTerminal({
        name: `FPAS: ${path.basename(target.fsPath)}`,
        shellPath: cli,
        shellArgs: runArguments(target.fsPath, standardLibrary, programArguments),
        cwd: path.dirname(target.fsPath),
        isTransient: true
      });
      this.status.text = "$(play) FPAS: Run";
      const closed = vscode.window.onDidCloseTerminal((value) => {
        if (value === terminal) {
          closed.dispose();
          void this.updateStatus();
        }
      });
      terminal.show();
    } catch (error) {
      await this.reportUnavailable(error);
    }
  }

  private async updateStatus(): Promise<void> {
    const selected = await this.selector.current();
    this.status.text =
      selected === undefined
        ? "$(tools) FPAS: Select project"
        : `$(tools) FPAS: ${path.basename(selected.fsPath)}`;
    this.status.tooltip = selected?.fsPath ?? "Select the active Functional Pascal project";
  }

  private async reportUnavailable(error: unknown): Promise<void> {
    const message = error instanceof Error ? error.message : String(error);
    this.output.appendLine(message);
    this.output.show(true);
    if (this.unavailableMessage !== message) {
      this.unavailableMessage = message;
      await vscode.window.showErrorMessage(message);
    }
  }
}

function operationLabel(operation: WorkflowOperation): string {
  switch (operation) {
    case "check":
      return "Check";
    case "build":
      return "Build";
    case "test":
      return "Test";
    case "format":
      return "Format";
    case "formatCheck":
      return "Format check";
  }
}

/** Parses a non-interactive JSON argument array for terminal runs. */
export function parseProgramArguments(input: string): string[] {
  const value: unknown = JSON.parse(input);
  if (!Array.isArray(value) || !value.every((argument) => typeof argument === "string")) {
    throw new Error("Program arguments must be a JSON array of strings.");
  }
  return value;
}

function validateProgramArguments(input: string): string | undefined {
  try {
    parseProgramArguments(input);
    return undefined;
  } catch (error) {
    return error instanceof Error ? error.message : String(error);
  }
}

function renderArgument(value: string): string {
  return /\s/u.test(value) ? JSON.stringify(value) : value;
}
