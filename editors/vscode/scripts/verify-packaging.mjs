import assert from "node:assert/strict";
import {
  mkdir,
  mkdtemp,
  readdir,
  rm,
  writeFile
} from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  clearStagedServers,
  stageServer
} from "./package/stage-server.mjs";

/** Verifies stale-target cleanup and single-target native staging. */
export async function verifyPackaging() {
  const temporaryRoot = await mkdtemp(
    path.join(os.tmpdir(), "fpas-package-contract-")
  );
  try {
    const extensionRoot = path.join(temporaryRoot, "extension");
    const staleDirectory = path.join(
      extensionRoot,
      "server",
      "other-target"
    );
    const sourcePath = path.join(temporaryRoot, "fpas-lsp.test");
    await writeFile(sourcePath, "native-server-fixture");
    await mkdir(staleDirectory, { recursive: true });
    await writeFile(path.join(staleDirectory, "stale.test"), "stale");

    await clearStagedServers(extensionRoot);
    const staged = await stageServer({
      extensionRoot,
      sourcePath,
      hostTarget: "test-x64",
      executableName: "fpas-lsp.test",
      platform: "win32"
    });
    assert.equal(
      staged,
      path.join(
        extensionRoot,
        "server",
        "test-x64",
        "fpas-lsp.test"
      )
    );
    assert.deepEqual(
      await readdir(path.join(extensionRoot, "server")),
      ["test-x64"]
    );
  } finally {
    await rm(temporaryRoot, { recursive: true, force: true });
  }
  console.log("Phase 7 staging contract verification passed.");
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  await verifyPackaging();
}
