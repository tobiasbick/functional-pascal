import { chmod, copyFile, mkdir, rm, stat } from "node:fs/promises";
import path from "node:path";

function generatedServerRoot(extensionRoot) {
  const root = path.resolve(extensionRoot, "server");
  const relative = path.relative(path.resolve(extensionRoot), root);
  if (relative !== "server") {
    throw new Error(`Refusing to clean unexpected server directory: ${root}`);
  }
  return root;
}

/** Removes every previously staged host server directory. */
export async function clearStagedServers(extensionRoot) {
  await rm(generatedServerRoot(extensionRoot), {
    recursive: true,
    force: true
  });
}

/** Stages exactly one native server built for the current host. */
export async function stageServer({
  extensionRoot,
  sourcePath,
  hostTarget,
  executableName,
  platform
}) {
  const source = await stat(sourcePath).catch(() => undefined);
  if (source === undefined || !source.isFile() || source.size === 0) {
    throw new Error(
      `Release language server was not created at ${sourcePath}. Run \`cargo build --release -p fpas-lsp\` from the repository root.`
    );
  }

  const serverRoot = generatedServerRoot(extensionRoot);
  const targetDirectory = path.join(serverRoot, hostTarget);
  const destination = path.join(targetDirectory, executableName);
  await mkdir(targetDirectory, { recursive: true });
  await copyFile(sourcePath, destination);
  if (platform !== "win32") {
    await chmod(destination, 0o755);
  }

  const staged = await stat(destination);
  if (!staged.isFile() || staged.size !== source.size) {
    throw new Error(`Staged language server is incomplete: ${destination}`);
  }
  return destination;
}
