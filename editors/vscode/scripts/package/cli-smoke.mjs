import assert from "node:assert/strict";
import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  rmSync,
  writeFileSync
} from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";

import AdmZip from "adm-zip";

/** Exercises the CLI and standard library extracted from the packaged VSIX. */
export function smokePackagedCli({
  vsixPath,
  hostTarget,
  executableName,
  platform
}) {
  const temporaryRoot = mkdtempSync(path.join(os.tmpdir(), "fpas packaged cli "));
  try {
    const archive = new AdmZip(vsixPath);
    const cliEntry = `extension/cli/${hostTarget}/${executableName}`;
    const cli = path.join(temporaryRoot, "bin", executableName);
    mkdirSync(path.dirname(cli), { recursive: true });
    writeFileSync(cli, archive.readFile(cliEntry));
    if (platform !== "win32") {
      chmodSync(cli, 0o755);
    }
    const standardLibrary = path.join(temporaryRoot, "standard-library");
    for (const entry of archive
      .getEntries()
      .filter((value) => value.entryName.startsWith("extension/standard-library/") && !value.isDirectory)) {
      const relative = entry.entryName.slice("extension/standard-library/".length);
      const destination = path.join(standardLibrary, ...relative.split("/"));
      mkdirSync(path.dirname(destination), { recursive: true });
      writeFileSync(destination, entry.getData());
    }

    const version = invoke(cli, ["--version"], temporaryRoot);
    assert.equal(version.status, 0, version.stderr);
    assert.match(version.stdout, /fpas \d+\.\d+\.\d+/u);

    const projectRoot = path.join(temporaryRoot, "project with spaces");
    mkdirSync(path.join(projectRoot, "src"), { recursive: true });
    const manifest = path.join(projectRoot, "smoke.fpasprj");
    writeFileSync(
      manifest,
      '[project]\nname = "smoke"\nkind = "program"\nmain = "src/main.fpas"\n\n[sources]\ninclude = ["src/**/*.fpas"]\n'
    );
    writeFileSync(
      path.join(projectRoot, "src", "main.fpas"),
      "program Smoke;\n\nbegin\nend.\n"
    );
    const checked = invoke(
      cli,
      ["check", "--std-lib", standardLibrary, manifest],
      projectRoot
    );
    assert.equal(checked.status, 0, checked.stderr);

    const testFile = path.join(projectRoot, "src", "smoke_test.fpas");
    writeFileSync(
      testFile,
      "program SmokeTest;\n\nuses Std.Test;\n\nbegin\n  AssertTrue(true)\nend.\n"
    );
    const tested = invoke(
      cli,
      ["test", "--std-lib", standardLibrary, "--report", "json", testFile],
      projectRoot
    );
    assert.equal(tested.status, 0, tested.stderr);
    assert.equal(JSON.parse(tested.stdout).tests[0].status, "pass");

    const commands = path.join(projectRoot, "debug.jsonl");
    writeFileSync(
      commands,
      '{"type":"request","id":1,"command":"initialize","arguments":{}}\n{"type":"request","id":2,"command":"launch","arguments":{"stop_on_entry":false}}\n'
    );
    const debugged = invoke(
      cli,
      ["debug", manifest, "--protocol", "jsonl", "--commands", commands, "--report", "jsonl", "--std-lib", standardLibrary],
      projectRoot
    );
    assert.equal(debugged.status, 0, debugged.stderr);
    const debugRecords = debugged.stdout.trim().split(/\r?\n/u).map(JSON.parse);
    assert.ok(debugRecords.some((record) => record.event === "terminated"));
  } finally {
    rmSync(temporaryRoot, { recursive: true, force: true });
  }
  console.log("Packaged CLI check, test, and debugger verification passed.");
}

function invoke(executable, args, cwd) {
  const result = spawnSync(executable, args, {
    cwd,
    encoding: "utf8",
    windowsHide: true
  });
  if (result.error) {
    throw result.error;
  }
  return result;
}
