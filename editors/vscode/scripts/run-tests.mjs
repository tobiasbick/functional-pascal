import path from "node:path";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readdirSync, rmSync } from "node:fs";
import os from "node:os";
import { fileURLToPath } from "node:url";

import { downloadAndUnzipVSCode, runTests as runExtensionTests } from "@vscode/test-electron";

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

async function removeTemporaryDirectory(directory) {
  const resolved = path.resolve(directory);
  const temporaryRoot = path.resolve(os.tmpdir());
  if (!resolved.startsWith(`${temporaryRoot}${path.sep}`)) {
    throw new Error(`Refusing to remove a directory outside the temporary root: ${resolved}`);
  }
  for (let attempt = 0; attempt < 40; attempt += 1) {
    try {
      rmSync(resolved, { recursive: true, force: true });
      return;
    } catch (error) {
      if (!["EPERM", "EBUSY", "ENOTEMPTY"].includes(error.code) || attempt === 39) throw error;
      await new Promise(resolve => setTimeout(resolve, 500));
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
  const restricted = process.argv.includes("--restricted");
  runCommand("cargo", ["build", "-p", "fpas-cli"]);
  runCommand(process.execPath, [path.join(extensionRoot, "scripts", "compile.mjs")]);
  cleanupTransientFixtures();
  const userDataDirectory = mkdtempSync(path.join(os.tmpdir(), "fpas-vscode-test-"));
  const restrictedDirectory = restricted
    ? mkdtempSync(path.join(os.tmpdir(), "fpas-vscode-restricted-"))
    : undefined;
  try {
    await verifyManifest();
    await verifyContracts();
    await verifyGrammar();
    const version = process.env.FPAS_VSCODE_TEST_VERSION ?? "1.137.0";
    const testPath = path.join(extensionRoot, "out", "test",
      restricted ? "restricted.test.js" : "extension.test.js");
    const environment = {
      ...process.env,
      PATH: `${path.resolve(extensionRoot, "..", "..", "target", "debug")}${path.delimiter}${process.env.PATH ?? ""}`
    };
    if (restricted) {
      const executable = await downloadAndUnzipVSCode({ version });
      const result = spawnSync(executable, [
        restrictedDirectory,
        "--no-sandbox", "--disable-gpu-sandbox", "--disable-updates",
        "--skip-welcome", "--skip-release-notes", "--disable-extensions",
        `--extensionDevelopmentPath=${extensionRoot}`,
        `--extensionTestsPath=${testPath}`,
        `--user-data-dir=${userDataDirectory}`
      ], { cwd: extensionRoot, stdio: "inherit", env: environment });
      if (result.error) throw result.error;
      if (result.status !== 0) throw new Error(`Restricted Mode host exited with status ${result.status}`);
    } else await runExtensionTests({
      version,
      extensionDevelopmentPath: extensionRoot,
      extensionTestsPath: testPath,
      launchArgs: [fixtureRoot, "--disable-extensions", "--disable-workspace-trust",
        `--user-data-dir=${userDataDirectory}`],
      extensionTestsEnv: environment
    });
  } finally {
    try {
      await removeTemporaryDirectory(userDataDirectory);
    } finally {
      if (restrictedDirectory !== undefined) {
        await removeTemporaryDirectory(restrictedDirectory);
      }
      cleanupTransientFixtures();
    }
  }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  await runTests();
}
