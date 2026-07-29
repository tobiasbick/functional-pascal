import { rm } from "node:fs/promises";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import { build } from "esbuild";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const extensionRoot = path.resolve(scriptDirectory, "..");

await rm(path.join(extensionRoot, "out"), { recursive: true, force: true });

const typeScript = path.join(
  extensionRoot,
  "node_modules",
  "typescript",
  "bin",
  "tsc"
);
const typeCheck = spawnSync(process.execPath, [typeScript], {
  cwd: extensionRoot,
  encoding: "utf8",
  stdio: "inherit"
});
if (typeCheck.error) {
  throw typeCheck.error;
}
if (typeCheck.status !== 0) {
  throw new Error(`TypeScript compilation exited with status ${typeCheck.status}`);
}

await build({
  entryPoints: [path.join(extensionRoot, "src", "extension.ts")],
  outfile: path.join(extensionRoot, "out", "src", "extension.js"),
  bundle: true,
  external: ["vscode"],
  format: "cjs",
  platform: "node",
  target: "node18",
  sourcemap: false,
  logLevel: "warning"
});
