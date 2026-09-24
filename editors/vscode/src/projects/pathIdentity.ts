import path from "node:path";

/** Stable local path identity: Windows is case-insensitive; POSIX preserves case. */
export function pathIdentity(value: string, platform: NodeJS.Platform = process.platform): string {
  const normalized = (platform === "win32" ? path.win32 : path.posix).normalize(value);
  return platform === "win32" ? normalized.toLowerCase() : normalized;
}
