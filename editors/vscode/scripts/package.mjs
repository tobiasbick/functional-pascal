import { mkdir, readFile, rm, stat } from "node:fs/promises";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { runTests } from "./run-tests.mjs";
import { verifyPackage } from "./verify-package.mjs";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const extensionRoot = path.resolve(scriptDirectory, "..");
const distDirectory = path.join(extensionRoot, "dist");
const manifest = JSON.parse(
  await readFile(path.join(extensionRoot, "package.json"), "utf8")
);
const outputPath = path.join(
  distDirectory,
  `functional-pascal-${manifest.version}.vsix`
);

function printHelp() {
  console.log(`Usage:
  npm run package --prefix editors/vscode

Builds, tests, packages, and verifies one platform-independent Functional Pascal VSIX.
The installed extension resolves an FPAS toolchain from its executable setting or PATH.

Output:
  editors/vscode/dist/functional-pascal-<version>.vsix

The command is non-interactive and does not publish.`);
}

function runCommand(command, args) {
  const result = spawnSync(command, args, {
    cwd: extensionRoot,
    encoding: "utf8",
    stdio: "inherit"
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(
      `Command failed with status ${result.status}: ${command} ${args.join(" ")}`
    );
  }
}

const arguments_ = process.argv.slice(2);
if (arguments_.includes("--help") || arguments_.includes("-h")) {
  printHelp();
  process.exit(0);
}
if (arguments_.length > 0) {
  throw new Error(
    `Unknown package argument: ${arguments_[0]}\nRun \`npm run package --prefix editors/vscode -- --help\` for usage.`
  );
}

if (path.relative(extensionRoot, distDirectory) !== "dist") {
  throw new Error(`Refusing to clean unexpected output directory: ${distDirectory}`);
}
await rm(distDirectory, { recursive: true, force: true });
await mkdir(distDirectory, { recursive: true });
await runTests();

const vsceScript = path.join(
  extensionRoot,
  "node_modules",
  "@vscode",
  "vsce",
  "vsce"
);
runCommand(process.execPath, [
  vsceScript,
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
console.log(`Built and verified toolchain-independent VSIX: ${outputPath}`);
