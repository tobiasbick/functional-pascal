import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const extensionRoot = path.resolve(scriptDirectory, "..");

/** Validates the bootstrap extension manifest. */
export async function verifyManifest() {
  const manifestPath = path.join(extensionRoot, "package.json");
  const manifest = JSON.parse(await readFile(manifestPath, "utf8"));

  assert.equal(
    `${manifest.publisher}.${manifest.name}`,
    "functional-pascal.functional-pascal"
  );
  assert.equal(manifest.version, "0.0.1");
  assert.equal(manifest.main, "./out/src/extension.js");
  assert.equal(manifest.engines?.vscode, "^1.85.0");
  assert.equal(manifest.scripts?.package, "node scripts/package.mjs");
  assert.equal(manifest.scripts?.publish, undefined);

  const commands = manifest.contributes?.commands ?? [];
  assert.deepEqual(commands, [
    {
      command: "functionalPascal.showOutput",
      title: "Show Output",
      category: "Functional Pascal"
    }
  ]);

  assert.deepEqual(manifest.activationEvents, [
    "onCommand:functionalPascal.showOutput"
  ]);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  await verifyManifest();
  console.log("Manifest verification passed.");
}
