const SUPPORTED_HOST_TARGETS = new Set([
  "win32-x64",
  "win32-arm64",
  "linux-x64",
  "linux-arm64",
  "darwin-x64",
  "darwin-arm64"
]);

/** Returns every host target supported by the local packaging contract. */
export function supportedHostTargets() {
  return [...SUPPORTED_HOST_TARGETS].sort();
}

/** Maps a Node desktop host to the matching VS Code package target. */
export function resolveHostTarget(platform, architecture) {
  const target = `${platform}-${architecture}`;
  if (!SUPPORTED_HOST_TARGETS.has(target)) {
    throw new Error(
      `Unsupported Functional Pascal VSIX host target: ${target}. Build on Windows, Linux, or macOS using an x64 or arm64 host.`
    );
  }
  return target;
}

/** Returns the native server filename for a supported host platform. */
export function serverExecutableName(platform) {
  return platform === "win32" ? "fpas-lsp.exe" : "fpas-lsp";
}

/** Returns the native CLI filename for a supported host platform. */
export function cliExecutableName(platform) {
  return platform === "win32" ? "fpas.exe" : "fpas";
}
