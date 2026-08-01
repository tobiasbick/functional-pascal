import { chmod, copyFile, mkdir, rm, stat } from "node:fs/promises";
import path from "node:path";

function generatedCliRoot(extensionRoot) {
  const root = path.resolve(extensionRoot, "cli");
  if (path.relative(path.resolve(extensionRoot), root) !== "cli") {
    throw new Error(`Refusing to clean unexpected CLI directory: ${root}`);
  }
  return root;
}

/** Removes every previously staged host CLI directory. */
export async function clearStagedCli(extensionRoot) {
  await rm(generatedCliRoot(extensionRoot), { recursive: true, force: true });
}

/** Stages exactly one native CLI built for the current host. */
export async function stageCli({
  extensionRoot,
  sourcePath,
  hostTarget,
  executableName,
  platform
}) {
  const source = await stat(sourcePath).catch(() => undefined);
  if (source === undefined || !source.isFile() || source.size === 0) {
    throw new Error(
      `Release Functional Pascal CLI was not created at ${sourcePath}. Run \`cargo build --release -p fpas-cli\` from the repository root.`
    );
  }
  const targetDirectory = path.join(generatedCliRoot(extensionRoot), hostTarget);
  const destination = path.join(targetDirectory, executableName);
  await mkdir(targetDirectory, { recursive: true });
  await copyFile(sourcePath, destination);
  if (platform !== "win32") {
    await chmod(destination, 0o755);
  }
  const staged = await stat(destination);
  if (!staged.isFile() || staged.size !== source.size) {
    throw new Error(`Staged Functional Pascal CLI is incomplete: ${destination}`);
  }
  return destination;
}
