import { mkdir, rm, stat } from "node:fs/promises";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { runTests } from "./run-tests.mjs";
import { verifyPackage } from "./verify-package.mjs";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const extensionRoot = path.resolve(scriptDirectory, "..");
const distDirectory = path.join(extensionRoot, "dist");
const outputPath = path.join(
  distDirectory,
  "functional-pascal-0.0.1-bootstrap.vsix"
);

function runVsce(args) {
  const vsceScript = path.join(
    extensionRoot,
    "node_modules",
    "@vscode",
    "vsce",
    "vsce"
  );
  const result = spawnSync(process.execPath, [vsceScript, ...args], {
    cwd: extensionRoot,
    encoding: "utf8",
    stdio: "inherit"
  });

  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    throw new Error(`vsce exited with status ${result.status}`);
  }
}

await runTests();
await mkdir(distDirectory, { recursive: true });
await rm(outputPath, { force: true });

runVsce([
  "package",
  "--no-dependencies",
  "--out",
  outputPath
]);

verifyPackage(outputPath);

const output = await stat(outputPath);
if (!output.isFile() || output.size === 0) {
  throw new Error(`VSIX was not created: ${outputPath}`);
}

console.log(`Built and verified: ${outputPath}`);
