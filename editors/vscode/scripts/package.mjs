import { mkdir, readFile, rm, stat } from "node:fs/promises";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import {
  resolveHostTarget,
  serverExecutableName
} from "./package/host.mjs";
import { smokePackagedServer } from "./package/lsp-smoke.mjs";
import {
  clearStagedServers,
  stageServer
} from "./package/stage-server.mjs";
import {
  clearStagedStandardLibrary,
  stageStandardLibrary
} from "./package/stage-standard-library.mjs";
import { runTests } from "./run-tests.mjs";
import { verifyPackage } from "./verify-package.mjs";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const extensionRoot = path.resolve(scriptDirectory, "..");
const repositoryRoot = path.resolve(extensionRoot, "..", "..");
const distDirectory = path.join(extensionRoot, "dist");
const manifest = JSON.parse(
  await readFile(path.join(extensionRoot, "package.json"), "utf8")
);
const hostTarget = resolveHostTarget(process.platform, process.arch);
const executableName = serverExecutableName(process.platform);
const outputPath = path.join(
  distDirectory,
  `functional-pascal-${manifest.version}-${hostTarget}.vsix`
);

function printHelp() {
  console.log(`Usage:
  npm run package --prefix editors/vscode

Builds, tests, stages, packages, and verifies one host-native Functional Pascal VSIX.

Output:
  editors/vscode/dist/functional-pascal-<version>-<host-target>.vsix

The command is non-interactive, does not publish, and replaces stale staged targets.`);
}

function runCommand(command, args, cwd) {
  const result = spawnSync(command, args, {
    cwd,
    encoding: "utf8",
    stdio: "inherit"
  });

  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    throw new Error(
      `Command failed with status ${result.status}: ${command} ${args.join(" ")}`
    );
  }
}

function cargoTargetDirectory() {
  const result = spawnSync(
    "cargo",
    ["metadata", "--format-version", "1", "--no-deps"],
    {
      cwd: repositoryRoot,
      encoding: "utf8",
      windowsHide: true
    }
  );
  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    throw new Error(
      `Cannot locate the Cargo target directory: ${result.stderr.trim()}`
    );
  }
  return JSON.parse(result.stdout).target_directory;
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

const relativeDist = path.relative(extensionRoot, distDirectory);
if (relativeDist !== "dist") {
  throw new Error(`Refusing to clean unexpected output directory: ${distDirectory}`);
}
await rm(distDirectory, { recursive: true, force: true });
await mkdir(distDirectory, { recursive: true });
await clearStagedServers(extensionRoot);
await clearStagedStandardLibrary(extensionRoot);
await runTests();
runCommand(
  "cargo",
  ["build", "--release", "-p", "fpas-lsp"],
  repositoryRoot
);
const releaseServer = path.join(
  cargoTargetDirectory(),
  "release",
  executableName
);
await stageServer({
  extensionRoot,
  sourcePath: releaseServer,
  hostTarget,
  executableName,
  platform: process.platform
});
await stageStandardLibrary({
  extensionRoot,
  sourceRoot: path.join(repositoryRoot, "lib")
});

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
  "--target",
  hostTarget,
  "--ignore-other-target-folders",
  "--no-dependencies",
  "--out",
  outputPath
], extensionRoot);

verifyPackage(outputPath, { hostTarget, executableName });
await smokePackagedServer({
  vsixPath: outputPath,
  hostTarget,
  executableName,
  platform: process.platform
});

const output = await stat(outputPath);
if (!output.isFile() || output.size === 0) {
  throw new Error(`VSIX was not created: ${outputPath}`);
}

console.log(`Built and verified ${hostTarget} VSIX: ${outputPath}`);
