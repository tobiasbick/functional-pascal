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
import {
  clearStagedStandardLibrary,
  stageStandardLibrary
} from "./package/stage-standard-library.mjs";

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
    const sourceLibrary = path.join(temporaryRoot, "lib");
    await writeFile(sourcePath, "native-server-fixture");
    await mkdir(path.join(sourceLibrary, "Std"), { recursive: true });
    await writeFile(
      path.join(sourceLibrary, "stdlib.fpasprj"),
      "[project]\nname = \"test\"\nkind = \"library\"\n"
    );
    await writeFile(path.join(sourceLibrary, "Std", "Sample.fpas"), "unit Std.Sample;\n");
    await writeFile(path.join(sourceLibrary, "Std", "Sample.fpascu"), "derived");
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
    await clearStagedStandardLibrary(extensionRoot);
    const stagedSources = await stageStandardLibrary({
      extensionRoot,
      sourceRoot: sourceLibrary
    });
    assert.deepEqual(stagedSources, [
      "stdlib.fpasprj",
      path.join("Std", "Sample.fpas")
    ]);
    assert.deepEqual(
      await readdir(path.join(extensionRoot, "standard-library", "Std")),
      ["Sample.fpas"]
    );
  } finally {
    await rm(temporaryRoot, { recursive: true, force: true });
  }
  console.log("Phase 7 staging contract verification passed.");
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  await verifyPackaging();
}
