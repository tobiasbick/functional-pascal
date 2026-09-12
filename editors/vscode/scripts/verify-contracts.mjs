import { readFile, readdir } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

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
  const cliPathSource = await readFile(
    path.join(extensionRoot, "src", "cliPath.ts"),
    "utf8"
  );
  assert(
    cliPathSource.includes(contract.toolchainPolicy.configuration) &&
      cliPathSource.includes("process.env.PATH"),
    "Toolchain lookup must prefer the configured executable and otherwise search PATH"
  );
  const toolchainSource = await readFile(
    path.join(extensionRoot, "src", "toolchain.ts"),
    "utf8"
  );
  assert(
    toolchainSource.includes('execFileAsync(executable, ["env", "--json"]'),
    "Toolchain validation must use the machine-readable environment command"
  );

  assert(
    contract.toolchainPolicy.fallback === "PATH" &&
      contract.toolchainPolicy.languageServerCommand[0] === "lsp",
    "The extension must use one installed toolchain for CLI and LSP"
  );
  assert(
    contract.toolchainPolicy.remoteHost === "unsupported"
      && contract.toolchainPolicy.remoteHostMessage.includes("same local"),
    "Remote-host rejection needs an actionable local-workspace message"
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
