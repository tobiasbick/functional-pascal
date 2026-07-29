import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { runTests as runExtensionTests } from "@vscode/test-electron";

import { verifyManifest } from "./verify-manifest.mjs";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const extensionRoot = path.resolve(scriptDirectory, "..");

function runNode(relativeScript, args = []) {
  const script = path.join(extensionRoot, relativeScript);
  const result = spawnSync(process.execPath, [script, ...args], {
    cwd: extensionRoot,
    encoding: "utf8",
    stdio: "inherit"
  });

  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    throw new Error(`${relativeScript} exited with status ${result.status}`);
  }
}

/** Runs compilation, manifest checks, and extension-host tests. */
export async function runTests() {
  runNode("node_modules/typescript/bin/tsc");
  await verifyManifest();
  await runExtensionTests({
    extensionDevelopmentPath: extensionRoot,
    extensionTestsPath: path.join(
      extensionRoot,
      "out",
      "test",
      "extension.test.js"
    ),
    launchArgs: [
      "--disable-extensions",
      "--disable-workspace-trust"
    ]
  });
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  await runTests();
}
