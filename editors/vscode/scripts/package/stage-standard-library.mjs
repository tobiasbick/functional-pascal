import { copyFile, mkdir, readdir, rm, stat } from "node:fs/promises";
import path from "node:path";

function generatedStandardLibraryRoot(extensionRoot) {
  const root = path.resolve(extensionRoot, "standard-library");
  const relative = path.relative(path.resolve(extensionRoot), root);
  if (relative !== "standard-library") {
    throw new Error(
      `Refusing to clean unexpected standard-library directory: ${root}`
    );
  }
  return root;
}

/** Removes a previously staged source standard library. */
export async function clearStagedStandardLibrary(extensionRoot) {
  await rm(generatedStandardLibraryRoot(extensionRoot), {
    recursive: true,
    force: true
  });
}

/** Stages the authoritative manifest and `.fpas` sources without derived artifacts. */
export async function stageStandardLibrary({ extensionRoot, sourceRoot }) {
  const manifest = path.join(sourceRoot, "stdlib.fpasprj");
  const manifestStatus = await stat(manifest).catch(() => undefined);
  if (manifestStatus === undefined || !manifestStatus.isFile()) {
    throw new Error(
      `Source standard-library manifest was not found at ${manifest}.`
    );
  }

  const destinationRoot = generatedStandardLibraryRoot(extensionRoot);
  await mkdir(destinationRoot, { recursive: true });
  const relativeSources = await collectSourceFiles(sourceRoot);
  for (const relativeSource of relativeSources) {
    const destination = path.join(destinationRoot, relativeSource);
    await mkdir(path.dirname(destination), { recursive: true });
    await copyFile(path.join(sourceRoot, relativeSource), destination);
  }
  return relativeSources;
}

async function collectSourceFiles(sourceRoot) {
  const collected = ["stdlib.fpasprj"];

  async function visit(relativeDirectory) {
    const directory = path.join(sourceRoot, relativeDirectory);
    const entries = await readdir(directory, { withFileTypes: true });
    entries.sort((left, right) => left.name.localeCompare(right.name));
    for (const entry of entries) {
      const relativePath = path.join(relativeDirectory, entry.name);
      if (entry.isDirectory()) {
        await visit(relativePath);
      } else if (entry.isFile() && entry.name.toLowerCase().endsWith(".fpas")) {
        collected.push(relativePath);
      }
    }
  }

  await visit("");
  return collected;
}
