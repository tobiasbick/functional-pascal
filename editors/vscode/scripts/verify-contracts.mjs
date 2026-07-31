import { readFile, readdir } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  resolveHostTarget,
  supportedHostTargets
} from "./package/host.mjs";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const extensionRoot = path.resolve(scriptDirectory, "..");
const repositoryRoot = path.resolve(extensionRoot, "..", "..");
const fixtureRoot = path.join(extensionRoot, "test", "fixtures");

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

async function readJson(relativePath) {
  const text = await readFile(path.join(extensionRoot, relativePath), "utf8");
  return JSON.parse(text);
}

async function collectFixtureFiles(directory) {
  const files = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const entryPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      files.push(...await collectFixtureFiles(entryPath));
    } else if (
      [".fpas", ".fpasprj", ".fpasworkspace"].includes(path.extname(entry.name))
    ) {
      files.push(path.relative(fixtureRoot, entryPath).replaceAll("\\", "/"));
    }
  }
  return files.sort();
}

function assertSameFiles(actual, expected) {
  assert(
    JSON.stringify(actual) === JSON.stringify(expected),
    `Fixture index mismatch.\nExpected: ${expected.join(", ")}\nActual: ${actual.join(", ")}`
  );
}

function assertLocalExtensionHost(hostPolicy, remoteName) {
  if (remoteName !== undefined) {
    throw new Error(hostPolicy.remoteHostMessage);
  }
}

function assertFailureMessage(action, expectedMessage) {
  try {
    action();
  } catch (error) {
    assert(error instanceof Error, "Contract failure must throw an Error");
    assert(error.message === expectedMessage, `Unexpected contract error: ${error.message}`);
    return;
  }
  throw new Error(`Contract action did not fail: ${expectedMessage}`);
}

/** Verifies the Phase 1 protocol contract and fixture inventory. */
export async function verifyContracts() {
  const contract = await readJson("contracts/phase1.json");
  assert(contract.schemaVersion === 1, "Unsupported Phase 1 contract schema");
  assert(contract.protocol.version === "3.17", "LSP 3.17 must remain the protocol baseline");
  assert(contract.protocol.transport === "stdio", "The language server transport must be stdio");
  assert(
    contract.protocol.positionEncoding === "utf-16",
    "LSP positions must use UTF-16 code units"
  );
  assert(
    contract.protocol.textDocumentSync.change === "full",
    "Initial document synchronization must use full text"
  );

  const requiredMethods = new Set([
    "initialize",
    "initialized",
    "shutdown",
    "exit",
    "textDocument/didOpen",
    "textDocument/didChange",
    "textDocument/didSave",
    "textDocument/didClose",
    "textDocument/publishDiagnostics",
    "textDocument/formatting",
    "textDocument/documentSymbol",
    "textDocument/hover",
    "textDocument/definition",
    "textDocument/completion",
    "textDocument/references",
    "textDocument/prepareRename",
    "textDocument/rename"
  ]);
  const configuredMethods = new Set(
    contract.capabilities.flatMap((capability) => capability.methods)
  );
  for (const method of requiredMethods) {
    assert(configuredMethods.has(method), `Missing contracted LSP method: ${method}`);
  }
  for (const capability of contract.capabilities) {
    assert(
      typeof capability.serviceQuery === "string" && capability.serviceQuery.length > 0,
      `Missing language-service query for ${capability.feature}`
    );
  }

  assert(
    contract.transportLibrary.selected.crate === "tower-lsp-server"
      && contract.transportLibrary.selected.baselineVersion === "0.23.0",
    "The selected transport baseline must be tower-lsp-server 0.23.0"
  );
  assert(
    contract.transportLibrary.rejected.crate === "lsp-server"
      && contract.transportLibrary.rejected.reason.length > 20,
    "The rejected transport must retain an explicit reason"
  );

  for (const evidence of contract.sourceEvidence) {
    const source = await readFile(path.join(repositoryRoot, evidence.path), "utf8");
    for (const symbol of evidence.symbols) {
      assert(
        source.includes(symbol),
        `Contract evidence is stale: ${evidence.path} no longer contains ${symbol}`
      );
    }
  }
  const serverPathSource = await readFile(
    path.join(extensionRoot, "src", "serverPath.ts"),
    "utf8"
  );
  assert(
    /"target",\s*"debug"/u.test(serverPathSource) &&
      /"server",/u.test(serverPathSource),
    "Server lookup must retain explicit development and packaged paths"
  );
  assert(
    !/(?:process\.env\.PATH|execFile|spawnSync|which|where\.exe)/u.test(
      serverPathSource
    ),
    "Server lookup must never search the system PATH"
  );

  assert(
    contract.hostPolicy.localTargets.length > 0,
    "At least one local native host target must be contracted"
  );
  assertSameFiles(
    supportedHostTargets(),
    [...contract.hostPolicy.localTargets].sort()
  );
  assert(
    contract.hostPolicy.unsupportedTargetMessage.includes("{platform}-{arch}")
      && contract.hostPolicy.unsupportedTargetMessage.includes("Build on"),
    "Unsupported host targets need an actionable build message"
  );
  assert(
    contract.hostPolicy.remoteHost === "unsupported"
      && contract.hostPolicy.remoteHostMessage.includes("Open the workspace"),
    "Remote-host rejection needs an actionable local-workspace message"
  );
  assert(
    resolveHostTarget("win32", "x64") === "win32-x64",
    "A supported local host target must resolve unchanged"
  );
  assertLocalExtensionHost(contract.hostPolicy, undefined);
  assertFailureMessage(
    () => resolveHostTarget("freebsd", "riscv64"),
    contract.hostPolicy.unsupportedTargetMessage
      .replace("{platform}", "freebsd")
      .replace("{arch}", "riscv64")
  );
  assertFailureMessage(
    () => assertLocalExtensionHost(contract.hostPolicy, "ssh-remote"),
    contract.hostPolicy.remoteHostMessage
  );

  const fixtureIndex = await readJson("test/fixtures/fixture-index.json");
  assert(fixtureIndex.schemaVersion === 1, "Unsupported fixture index schema");
  const coverage = new Set([
    ...fixtureIndex.sources.flatMap((source) => source.covers),
    ...fixtureIndex.workspace.covers
  ]);
  for (const requirement of fixtureIndex.requiredCoverage) {
    assert(coverage.has(requirement), `Missing fixture coverage: ${requirement}`);
  }

  for (const fixture of fixtureIndex.sources) {
    assert(fixture.path.endsWith(".fpas"), `Source fixture must be .fpas: ${fixture.path}`);
    const source = await readFile(path.join(fixtureRoot, fixture.path), "utf8");
    assert(source.length > 0, `Source fixture is empty: ${fixture.path}`);
    if (fixture.expected === "diagnostic") {
      assert(
        Array.isArray(fixture.diagnostics) && fixture.diagnostics.length > 0,
        `Diagnostic fixture has no expected code: ${fixture.path}`
      );
    }
  }

  const declaredFiles = [
    ...fixtureIndex.sources.map((source) => source.path),
    fixtureIndex.workspace.path,
    ...fixtureIndex.workspace.projects
  ].sort();
  assertSameFiles(await collectFixtureFiles(fixtureRoot), declaredFiles);

  console.log("Phase 1 contract and fixture verification passed.");
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  await verifyContracts();
}
