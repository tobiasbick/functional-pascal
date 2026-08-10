import assert from "node:assert/strict";
import {
  chmodSync,
  existsSync,
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
      "program Smoke;\n\nbegin\n  var Value: integer := 1\nend.\n"
    );
    const checked = invoke(
      cli,
      ["check", "--std-lib", standardLibrary, manifest],
      projectRoot
    );
    assert.equal(checked.status, 0, checked.stderr);

    const built = invoke(
      cli,
      ["build", "--std-lib", standardLibrary, manifest],
      projectRoot
    );
    assert.equal(built.status, 0, built.stderr);
    const image = path.join(projectRoot, "smoke.fpascp");
    assert.ok(existsSync(image), "packaged CLI produces a compiled program image");

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

    const dap = invoke(
      cli,
      [
        "debug",
        "--std-lib",
        standardLibrary,
        image,
        "--protocol",
        "dap",
        "--source-root",
        projectRoot
      ],
      projectRoot,
      frameDapRequests([
        dapRequest(1, "initialize", {}),
        dapRequest(2, "launch", { stopOnEntry: true }),
        dapRequest(3, "setBreakpoints", {
          source: { path: path.join(projectRoot, "src", "main.fpas") },
          breakpoints: [{ line: 4 }]
        }),
        dapRequest(4, "configurationDone", {}),
        dapRequest(5, "threads", {}),
        dapRequest(6, "stackTrace", { threadId: 1 }),
        dapRequest(7, "source", {
          source: { path: path.join(projectRoot, "src", "main.fpas") },
          sourceReference: 0
        }),
        dapRequest(8, "disconnect", { terminateDebuggee: true })
      ])
    );
    assert.equal(dap.status, 0, dap.stderr);
    const dapMessages = parseDapMessages(dap.stdout);
    for (const command of [
      "initialize",
      "launch",
      "setBreakpoints",
      "configurationDone",
      "threads",
      "stackTrace",
      "source",
      "disconnect"
    ]) {
      assert.ok(
        dapMessages.some(
          (message) =>
            message.type === "response" &&
            message.command === command &&
            message.success === true
        ),
        `packaged DAP request ${command} succeeds`
      );
    }
    assert.ok(
      dapMessages.some(
        (message) =>
          message.event === "stopped" && message.body?.reason === "entry"
      ),
      "packaged DAP stops on entry"
    );
    assert.ok(
      dapMessages.some(
        (message) =>
          message.command === "setBreakpoints" &&
          message.body?.breakpoints?.[0]?.verified === true
      ),
      "packaged DAP verifies source breakpoints"
    );
    assert.ok(
      dapMessages.some(
        (message) =>
          message.command === "source" &&
          message.body?.content?.includes("program Smoke;")
      ),
      "packaged DAP returns verified source content"
    );
  } finally {
    rmSync(temporaryRoot, { recursive: true, force: true });
  }
  console.log("Packaged CLI check, test, and debugger verification passed.");
}

function invoke(executable, args, cwd, input) {
  const result = spawnSync(executable, args, {
    cwd,
    encoding: "utf8",
    input,
    windowsHide: true
  });
  if (result.error) {
    throw result.error;
  }
  return result;
}

function dapRequest(seq, command, arguments_) {
  return { seq, type: "request", command, arguments: arguments_ };
}

function frameDapRequests(requests) {
  return requests.map((request) => {
    const body = JSON.stringify(request);
    return `Content-Length: ${Buffer.byteLength(body, "utf8")}\r\n\r\n${body}`;
  }).join("");
}

function parseDapMessages(output) {
  const bytes = Buffer.from(output, "utf8");
  const separator = Buffer.from("\r\n\r\n", "ascii");
  const messages = [];
  let offset = 0;
  while (offset < bytes.length) {
    const headerEnd = bytes.indexOf(separator, offset);
    assert.ok(headerEnd >= 0, "packaged DAP output has complete headers");
    const header = bytes.subarray(offset, headerEnd).toString("ascii");
    const match = /^Content-Length:\s*(\d+)$/imu.exec(header);
    assert.ok(match, `packaged DAP output has Content-Length: ${header}`);
    const length = Number(match[1]);
    const bodyStart = headerEnd + separator.length;
    const bodyEnd = bodyStart + length;
    assert.ok(bodyEnd <= bytes.length, "packaged DAP output has a complete body");
    messages.push(JSON.parse(bytes.subarray(bodyStart, bodyEnd).toString("utf8")));
    offset = bodyEnd;
  }
  return messages;
}
