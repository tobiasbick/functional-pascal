import assert from "node:assert/strict";
import path from "node:path";

import * as vscode from "vscode";

import { WorkflowTesting } from "../../src/workflow/testing";
import { ProjectSelector } from "../../src/workflow/project";
import { WorkflowProcessRunner } from "../../src/workflow/processes";
import type { WorkflowProcessResult, WorkflowTestStatus } from "../../src/workflow/model";

/** Exercises real Testing API include/exclude requests against nested report identities. */
export async function verifyTestingRequests(): Promise<void> {
  const directory = vscode.Uri.file(path.join(path.parse(process.cwd()).root, "testing requests")).fsPath;
  const manifest = vscode.Uri.file(path.join(directory, "suite.fpasprj"));
  const files = [
    path.join(directory, "a", "same_test.fpas"),
    path.join(directory, "b", "same_test.fpas"),
    path.join(directory, "a", "other_same_test.fpas")
  ];
  const requests: string[][] = [];
  const runner = {
    run: async (_executable: string, args: readonly string[]): Promise<WorkflowProcessResult> => {
      if (args.includes("--list")) {
        return { exitCode: 0, stdout: files.join("\n"), stderr: "", cancelled: false };
      }
      requests.push([...args]);
      const fileIndex = args.indexOf("--file");
      const selected = fileIndex >= 0 ? [args[fileIndex + 1]] : files;
      return {
        exitCode: 0,
        stdout: JSON.stringify({ version: 1,
          tests: selected.map(file => ({ file, status: file === files[1] ? "assert_failed" : "pass" })) }),
        stderr: "",
        cancelled: false
      };
    }
  } as unknown as WorkflowProcessRunner;
  const selector = { current: async () => manifest } as ProjectSelector;
  const testing = new WorkflowTesting(selector, runner, async () => "fpas", async () => undefined,
    () => undefined, "functionalPascalReviewRequests");
  const access = testing as unknown as {
    controller: vscode.TestController;
    lastStatuses: Map<string, WorkflowTestStatus>;
    runRequest(request: vscode.TestRunRequest, token: vscode.CancellationToken): Promise<void>;
  };
  const cancellation = new vscode.CancellationTokenSource();
  try {
    assert.deepEqual(await testing.discover(), files);
    let root: vscode.TestItem | undefined;
    access.controller.items.forEach(item => { root = item; });
    assert.ok(root);
    const items = files.map(file => {
      const item = root!.children.get(`test:${path.normalize(file)}`);
      assert.ok(item, file);
      return item;
    });
    await access.runRequest(new vscode.TestRunRequest([items[0]]), cancellation.token);
    assert.deepEqual(requests.map(args => args[args.indexOf("--file") + 1]), [files[0]]);
    assert.deepEqual(Object.fromEntries(access.lastStatuses), { [files[0]]: "pass" });

    requests.length = 0;
    await access.runRequest(new vscode.TestRunRequest(undefined, [items[1]]), cancellation.token);
    assert.deepEqual(requests.map(args => args[args.indexOf("--file") + 1]), [files[0], files[2]]);
    assert.deepEqual(Object.fromEntries(access.lastStatuses),
      { [files[0]]: "pass", [files[2]]: "pass" });

    requests.length = 0;
    await access.runRequest(new vscode.TestRunRequest([root], [items[0], items[2]]), cancellation.token);
    assert.deepEqual(requests.map(args => args[args.indexOf("--file") + 1]), [files[1]]);
    assert.deepEqual(Object.fromEntries(access.lastStatuses), { [files[1]]: "assert_failed" });

    requests.length = 0;
    await access.runRequest(new vscode.TestRunRequest(undefined, [root]), cancellation.token);
    assert.equal(requests.length, 0);
    assert.equal(access.lastStatuses.size, 0);

    requests.length = 0;
    await access.runRequest(new vscode.TestRunRequest(), cancellation.token);
    assert.equal(requests.length, 1);
    assert.equal(requests[0].includes("--file"), false);
    assert.equal(access.lastStatuses.size, 3);
  } finally {
    cancellation.dispose();
    testing.dispose();
  }
}
