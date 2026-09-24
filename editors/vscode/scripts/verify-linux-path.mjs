import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";

const require = createRequire(import.meta.url);
const { pathIdentity } = require("../out/src/projects/pathIdentity.js");

assert.equal(process.platform, "linux", "run this check under a Linux Node host");
const root = mkdtempSync(path.join(os.tmpdir(), "fpas-path-case-"));
try {
  const upper = path.join(root, "Main.fpas");
  const lower = path.join(root, "main.fpas");
  writeFileSync(upper, "upper");
  writeFileSync(lower, "lower");
  assert.notEqual(pathIdentity(upper), pathIdentity(lower));
  assert.equal(pathIdentity(path.join(root, ".", "Main.fpas")), pathIdentity(upper));
} finally {
  rmSync(root, { recursive: true, force: true });
}
console.log("Linux path identity check passed.");
