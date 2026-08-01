import path from "node:path";

import * as vscode from "vscode";

import { parseTestReport, parseWorkflowDiagnostics } from "./diagnostics";
import { testListArguments, testRunArguments } from "./arguments";
import type { WorkflowProcessResult, WorkflowTestStatus } from "./model";
import { WorkflowProcessRunner } from "./processes";
import { ProjectSelector } from "./project";

/** Testing API integration backed exclusively by `fpas test`. */
export class WorkflowTesting implements vscode.Disposable {
  private readonly controller = vscode.tests.createTestController(
    "functionalPascalTests",
    "Functional Pascal"
  );
  private readonly itemsByPath = new Map<string, vscode.TestItem>();
  private readonly lastStatuses = new Map<string, WorkflowTestStatus>();
  private commandCancellation: vscode.CancellationTokenSource | undefined;
  private readonly runProfile: vscode.TestRunProfile;

  public constructor(
    private readonly selector: ProjectSelector,
    private readonly runner: WorkflowProcessRunner,
    private readonly resolveCli: () => Promise<string>,
    private readonly resolveStandardLibrary: () => string,
    private readonly publishDiagnostics: (
      stderr: string,
      cwd: string
    ) => Promise<void>,
    private readonly setRunning: (running: boolean) => void
  ) {
    this.controller.refreshHandler = async () => {
      await this.discover();
    };
    this.runProfile = this.controller.createRunProfile(
      "Run",
      vscode.TestRunProfileKind.Run,
      async (request, token) => {
        await this.runRequest(request, token);
      },
      true
    );
  }

  /** Discovers tests for the selected project without prompting. */
  public async discover(): Promise<string[]> {
    this.controller.items.replace([]);
    this.itemsByPath.clear();
    const target = await this.selector.current();
    if (target === undefined) {
      return [];
    }
    const cwd = path.dirname(target.fsPath);
    const root = this.controller.createTestItem(
      `project:${target.toString()}`,
      path.basename(target.fsPath),
      target
    );
    this.controller.items.add(root);
    let result: WorkflowProcessResult;
    try {
      result = await this.runner.run(
        await this.resolveCli(),
        testListArguments(target.fsPath, this.resolveStandardLibrary()),
        cwd
      );
    } catch (error) {
      root.error = error instanceof Error ? error.message : String(error);
      return [];
    }
    if (result.exitCode !== 0) {
      root.error = result.stderr.trim() || "Functional Pascal test discovery failed.";
      return [];
    }
    const files = result.stdout
      .replaceAll("\r\n", "\n")
      .split("\n")
      .map((value) => value.trim())
      .filter((value) => value.length > 0)
      .map((value) => (path.isAbsolute(value) ? value : path.resolve(cwd, value)));
    for (const file of files) {
      const normalized = path.normalize(file);
      const item = this.controller.createTestItem(
        `test:${normalized}`,
        path.basename(normalized),
        vscode.Uri.file(normalized)
      );
      item.range = new vscode.Range(0, 0, 0, 0);
      root.children.add(item);
      this.itemsByPath.set(normalized.toLocaleLowerCase(), item);
    }
    return files;
  }

  /** Runs all or selected discovered cases through a real Testing API run. */
  public async runFiles(files?: readonly string[]): Promise<Record<string, WorkflowTestStatus>> {
    if (this.itemsByPath.size === 0) {
      await this.discover();
    }
    const include = files?.map((file) =>
      this.itemsByPath.get(path.normalize(file).toLocaleLowerCase())
    );
    const request = new vscode.TestRunRequest(
      include?.filter((item): item is vscode.TestItem => item !== undefined)
    );
    this.commandCancellation?.cancel();
    this.commandCancellation?.dispose();
    const cancellation = new vscode.CancellationTokenSource();
    this.commandCancellation = cancellation;
    try {
      await this.runRequest(request, cancellation.token);
    } finally {
      if (this.commandCancellation === cancellation) {
        this.commandCancellation = undefined;
      }
      cancellation.dispose();
    }
    return Object.fromEntries(this.lastStatuses);
  }

  /** Cancels a command-triggered test run. Testing UI runs use VS Code's token. */
  public cancel(): void {
    this.commandCancellation?.cancel();
  }

  public dispose(): void {
    this.cancel();
    this.commandCancellation?.dispose();
    this.runProfile.dispose();
    this.controller.dispose();
  }

  private async runRequest(
    request: vscode.TestRunRequest,
    token: vscode.CancellationToken
  ): Promise<void> {
    const target = await this.selector.current();
    const run = this.controller.createTestRun(request);
    this.setRunning(true);
    this.lastStatuses.clear();
    if (target === undefined) {
      run.end();
      this.setRunning(false);
      return;
    }
    const requested = this.requestedItems(request);
    for (const item of requested) {
      run.enqueued(item);
    }
    const batches =
      request.include === undefined
        ? [undefined]
        : requested.map((item) => item.uri?.fsPath);
    const cwd = path.dirname(target.fsPath);
    try {
      for (const file of batches) {
        if (token.isCancellationRequested) {
          break;
        }
        const batchItems =
          file === undefined
            ? requested
            : requested.filter((item) => item.uri?.fsPath === file);
        for (const item of batchItems) {
          run.started(item);
        }
        const result = await this.runner.run(
          await this.resolveCli(),
          testRunArguments(
            target.fsPath,
            this.resolveStandardLibrary(),
            file === undefined ? undefined : path.basename(file),
            vscode.workspace
              .getConfiguration("functionalPascal")
              .get<number>("testTimeoutSeconds", 10)
          ),
          cwd,
          token
        );
        run.appendOutput(`${result.stderr.replaceAll("\n", "\r\n")}\r\n`);
        await this.publishDiagnostics(result.stderr, cwd);
        if (result.cancelled) {
          for (const item of batchItems) {
            this.record(run, item, "not_run");
          }
          break;
        }
        const report = parseTestReport(result.stdout);
        for (const test of report.tests) {
          const absolute = path.isAbsolute(test.file)
            ? path.normalize(test.file)
            : path.resolve(cwd, test.file);
          const item = this.itemsByPath.get(absolute.toLocaleLowerCase());
          if (item !== undefined) {
            this.record(run, item, test.status);
          }
        }
      }
    } catch (error) {
      const message = new vscode.TestMessage(
        error instanceof Error ? error.message : String(error)
      );
      for (const item of requested) {
        if (!this.lastStatuses.has(item.uri?.fsPath ?? "")) {
          run.errored(item, message);
        }
      }
    } finally {
      if (token.isCancellationRequested) {
        for (const item of requested) {
          const key = item.uri?.fsPath ?? item.id;
          if (!this.lastStatuses.has(key)) {
            this.record(run, item, "not_run");
          }
        }
      }
      run.end();
      this.setRunning(false);
    }
  }

  private requestedItems(request: vscode.TestRunRequest): vscode.TestItem[] {
    if (request.include === undefined) {
      return [...this.itemsByPath.values()];
    }
    const selected = new Set<vscode.TestItem>();
    const visit = (item: vscode.TestItem): void => {
      if (item.uri?.fsPath.endsWith("_test.fpas") === true) {
        selected.add(item);
      }
      item.children.forEach(visit);
    };
    request.include.forEach(visit);
    return [...selected];
  }

  private record(
    run: vscode.TestRun,
    item: vscode.TestItem,
    status: WorkflowTestStatus
  ): void {
    const key = item.uri?.fsPath ?? item.id;
    this.lastStatuses.set(key, status);
    switch (status) {
      case "pass":
        run.passed(item);
        break;
      case "skipped":
      case "not_run":
        run.skipped(item);
        break;
      case "assert_failed":
        run.failed(item, new vscode.TestMessage("FPAS assertion failed."));
        break;
      case "compile_error":
        run.errored(item, new vscode.TestMessage("FPAS test did not compile."));
        break;
      case "runtime_error":
        run.errored(item, new vscode.TestMessage("FPAS test had a runtime error."));
        break;
      case "timed_out":
        run.errored(item, new vscode.TestMessage("FPAS test timed out."));
        break;
    }
  }
}
