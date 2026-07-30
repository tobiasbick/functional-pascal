import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { runTests as runExtensionTests } from "@vscode/test-electron";

import { verifyContracts } from "./verify-contracts.mjs";
import { verifyGrammar } from "./verify-grammar.mjs";
import { verifyManifest } from "./verify-manifest.mjs";
import { verifyPackaging } from "./verify-packaging.mjs";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const extensionRoot = path.resolve(scriptDirectory, "..");

function runCommand(command, args) {
  const result = spawnSync(command, args, {
    cwd: extensionRoot,
    encoding: "utf8",
    stdio: "inherit"
  });

  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    throw new Error(`${command} exited with status ${result.status}`);
  }
}

/** Runs compilation, manifest checks, and extension-host tests. */
export async function runTests() {
  runCommand("cargo", ["build", "-p", "fpas-lsp"]);
  runCommand(process.execPath, [path.join(extensionRoot, "scripts", "compile.mjs")]);
  await verifyManifest();
  await verifyContracts();
  await verifyPackaging();
  await verifyGrammar();
  await runExtensionTests({
    extensionDevelopmentPath: extensionRoot,
    extensionTestsPath: path.join(
      extensionRoot,
      "out",
      "test",
      "extension.test.js"
    ),
    launchArgs: [
      path.join(extensionRoot, "test", "fixtures"),
      "--disable-extensions",
      "--disable-workspace-trust"
    ]
  });
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  await runTests();
}
