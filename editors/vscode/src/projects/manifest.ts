import path from "node:path";

import * as toml from "@iarna/toml";

/** Return a program's absolute main path from a valid TOML project header. */
export function programMainPath(manifestPath: string, contents: string): string | undefined {
  const project = toml.parse(contents).project;
  if (project === null || typeof project !== "object" || Array.isArray(project)) {
    return undefined;
  }
  const fields = project as Record<string, unknown>;
  if (fields.kind !== "program" || typeof fields.main !== "string" || fields.main.trim() === "") {
    return undefined;
  }
  return path.resolve(path.dirname(manifestPath), fields.main);
}
