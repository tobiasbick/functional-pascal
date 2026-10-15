import { mkdir, readFile, rm, stat } from "node:fs/promises";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

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
  npm run package:verified --prefix editors/vscode

Compiles the extension, packages one platform-independent VSIX, and verifies its contents.
The default command requires Node.js 22 or newer and does not build FPAS or run tests.
The installed extension resolves an FPAS toolchain from its executable setting or PATH.

Options:
  --verified  Build FPAS and run the full extension test suite before packaging.
              Requires a stable Rust toolchain; packaging stops if a test fails.
  -h, --help  Show this help without building or testing.

Examples:
  npm run package --prefix editors/vscode
  npm run package:verified --prefix editors/vscode

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
const unknownArgument = arguments_.find((argument) => argument !== "--verified");
if (unknownArgument !== undefined) {
  throw new Error(
    `Unknown package argument: ${unknownArgument}\nRun \`npm run package --prefix editors/vscode -- --help\` for usage.`
  );
}

if (arguments_.includes("--verified")) {
  const { runTests } = await import("./run-tests.mjs");
  await runTests();
}

if (path.relative(extensionRoot, distDirectory) !== "dist") {
  throw new Error(`Refusing to clean unexpected output directory: ${distDirectory}`);
}
await rm(distDirectory, { recursive: true, force: true });
await mkdir(distDirectory, { recursive: true });

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
