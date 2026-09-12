import path from "node:path";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readdirSync, rmSync } from "node:fs";
import os from "node:os";
import { fileURLToPath } from "node:url";

import { runTests as runExtensionTests } from "@vscode/test-electron";

import { verifyContracts } from "./verify-contracts.mjs";
import { verifyGrammar } from "./verify-grammar.mjs";
import { verifyManifest } from "./verify-manifest.mjs";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const extensionRoot = path.resolve(scriptDirectory, "..");
const fixtureRoot = path.join(extensionRoot, "test", "fixtures");
const transientFixturePrefixes = [".project-index-", ".workspace-navigation-"];

function cleanupTransientFixtures() {
  for (const entry of readdirSync(fixtureRoot, { withFileTypes: true })) {
    if (
      entry.isDirectory() &&
      transientFixturePrefixes.some((prefix) => entry.name.startsWith(prefix))
    ) {
      rmSync(path.join(fixtureRoot, entry.name), { recursive: true, force: true });
    }
  }
}

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
  runCommand("cargo", ["build", "-p", "fpas-cli"]);
  runCommand(process.execPath, [path.join(extensionRoot, "scripts", "compile.mjs")]);
  cleanupTransientFixtures();
  const userDataDirectory = mkdtempSync(path.join(os.tmpdir(), "fpas-vscode-test-"));
  try {
    await verifyManifest();
    await verifyContracts();
    await verifyGrammar();
    await runExtensionTests({
      version: "1.137.0",
      extensionDevelopmentPath: extensionRoot,
      extensionTestsPath: path.join(
        extensionRoot,
        "out",
        "test",
        "extension.test.js"
      ),
      launchArgs: [fixtureRoot, "--disable-extensions", "--disable-workspace-trust",
        `--user-data-dir=${userDataDirectory}`],
      extensionTestsEnv: {
        ...process.env,
        PATH: `${path.resolve(extensionRoot, "..", "..", "target", "debug")}${path.delimiter}${process.env.PATH ?? ""}`
      }
    });
  } finally {
    rmSync(userDataDirectory, { recursive: true, force: true, maxRetries: 5, retryDelay: 200 });
    cleanupTransientFixtures();
  }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  await runTests();
}
