import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import AdmZip from "adm-zip";

import {
  cliExecutableName,
  resolveHostTarget,
  serverExecutableName
} from "./package/host.mjs";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const extensionRoot = path.resolve(scriptDirectory, "..");
const sourceManifest = JSON.parse(
  readFileSync(path.join(extensionRoot, "package.json"), "utf8")
);
const defaultHostTarget = resolveHostTarget(process.platform, process.arch);
const defaultVsixPath = path.join(
  extensionRoot,
  "dist",
  `functional-pascal-${sourceManifest.version}-${defaultHostTarget}.vsix`
);

/** Verifies that the local VSIX contains only intended runtime files. */
export function verifyPackage(vsixPath = defaultVsixPath, options = {}) {
  const hostTarget = options.hostTarget ?? defaultHostTarget;
  const executableName =
    options.executableName ?? serverExecutableName(process.platform);
  const cliName = options.cliName ?? cliExecutableName(process.platform);
  const serverEntry =
    `extension/server/${hostTarget}/${executableName}`;
  const cliEntry = `extension/cli/${hostTarget}/${cliName}`;
  const archive = new AdmZip(vsixPath);
  const entries = archive.getEntries().map((entry) => entry.entryName);
  const entrySet = new Set(entries);
  const standardLibraryEntries = collectFiles(
    path.join(extensionRoot, "standard-library")
  ).map(
    (relativePath) =>
      `extension/standard-library/${relativePath.replaceAll("\\", "/")}`
  );
  const expectedEntries = [
    "[Content_Types].xml",
    "extension.vsixmanifest",
    "extension/BUG_REPORT.md",
    cliEntry,
    "extension/LICENSE.txt",
    "extension/language-configuration.json",
    "extension/out/src/extension.js",
    "extension/package.json",
    "extension/readme.md",
    serverEntry,
    "extension/snippets/fpas.json",
    "extension/syntaxes/fpas.tmLanguage.json",
    ...standardLibraryEntries
  ].sort();
  assert.deepEqual(
    [...entries].sort(),
    expectedEntries,
    "VSIX contains exactly the intended runtime files"
  );

  for (const required of [
    "extension/package.json",
    "extension/BUG_REPORT.md",
    "extension/out/src/extension.js",
    "extension/readme.md",
    "extension/LICENSE.txt",
    "extension/language-configuration.json",
    "extension/snippets/fpas.json",
    "extension/syntaxes/fpas.tmLanguage.json",
    "extension/standard-library/stdlib.fpasprj",
    serverEntry,
    cliEntry
  ]) {
    assert.ok(entrySet.has(required), `VSIX contains ${required}`);
  }
  assert.ok(
    archive.readFile(serverEntry)?.length > 0,
    "packaged language server is non-empty"
  );
  assert.ok(
    archive.readFile(cliEntry)?.length > 0,
    "packaged Functional Pascal CLI is non-empty"
  );
  if (executableName === "fpas-lsp") {
    const serverMode = archive.getEntry(serverEntry).attr >>> 16;
    assert.ok(
      (serverMode & 0o111) !== 0,
      "packaged Unix language server is executable"
    );
  }
  if (cliName === "fpas") {
    const cliMode = archive.getEntry(cliEntry).attr >>> 16;
    assert.ok((cliMode & 0o111) !== 0, "packaged Unix CLI is executable");
  }

  const vsixManifest = archive.readAsText("extension.vsixmanifest");
  assert.match(
    vsixManifest,
    new RegExp(`TargetPlatform="${hostTarget}"`, "u"),
    "VSIX metadata identifies the current host target"
  );

  const packagedManifest = JSON.parse(
    archive.readAsText("extension/package.json")
  );
  assert.equal(
    packagedManifest.contributes.commands[0].command,
    "functionalPascal.showOutput"
  );
  assert.deepEqual(packagedManifest.contributes.languages[0].extensions, [
    ".fpas"
  ]);
  assert.equal(
    packagedManifest.contributes.languages[0].configuration,
    "./language-configuration.json"
  );
  assert.equal(
    packagedManifest.contributes.grammars[0].path,
    "./syntaxes/fpas.tmLanguage.json"
  );
  assert.deepEqual(packagedManifest.contributes.snippets, [
    { language: "fpas", path: "./snippets/fpas.json" }
  ]);
  assert.equal(
    packagedManifest.contributes.configuration.properties[
      "functionalPascal.testTimeoutSeconds"
    ].default,
    10
  );

  const packagedLanguageConfiguration = JSON.parse(
    archive.readAsText("extension/language-configuration.json")
  );
  assert.equal(packagedLanguageConfiguration.comments.lineComment, "//");

  const packagedGrammar = JSON.parse(
    archive.readAsText("extension/syntaxes/fpas.tmLanguage.json")
  );
  assert.equal(packagedGrammar.scopeName, "source.fpas");

  const packagedSnippets = JSON.parse(
    archive.readAsText("extension/snippets/fpas.json")
  );
  assert.ok(Object.keys(packagedSnippets).length >= 10);

  const compiledExtension = archive.readAsText(
    "extension/out/src/extension.js"
  );
  assert.ok(
    compiledExtension.includes(
      "Functional Pascal extension activated (Hello World)."
    ),
    "compiled extension contains the activation message"
  );
  assert.ok(
    compiledExtension.includes("functionalPascal.showOutput"),
    "compiled extension contains the output command"
  );
  assert.ok(
    compiledExtension.includes("functionalPascal.restartLanguageServer"),
    "compiled extension contains the restart command"
  );
  assert.ok(
    compiledExtension.includes("functionalPascal.checkProject") &&
      compiledExtension.includes("functionalPascal.testProject") &&
      compiledExtension.includes("functionalPascal.cancelOperation"),
    "compiled extension contains project workflow commands"
  );
  assert.ok(
    compiledExtension.includes("Content-Length"),
    "compiled extension bundles the stdio language client"
  );
  assert.ok(
    compiledExtension.includes("server") &&
      compiledExtension.includes("fpas-lsp"),
    "compiled extension resolves the packaged language server"
  );
  assert.ok(
    compiledExtension.includes("cli") && compiledExtension.includes("fpas"),
    "compiled extension resolves the packaged Functional Pascal CLI"
  );
  assert.ok(
    compiledExtension.includes("standardLibraryUri") &&
      compiledExtension.includes("standard-library"),
    "compiled extension resolves the packaged source standard library"
  );

  const forbiddenPrefixes = [
    "extension/src/",
    "extension/test/",
    "extension/scripts/",
    "extension/contracts/",
    "extension/node_modules/",
    "extension/out/test/",
    "extension/dist/",
    "extension/target/"
  ];

  for (const entry of entries) {
    assert.ok(!entry.endsWith(".map"), `VSIX excludes source map ${entry}`);
    assert.ok(
      !forbiddenPrefixes.some((prefix) => entry.startsWith(prefix)),
      `VSIX excludes development file ${entry}`
    );
    assert.ok(
      !/^[a-zA-Z]:[\\/]/u.test(entry) && !entry.startsWith("/"),
      `VSIX entry is relative: ${entry}`
    );
  }

  const serverEntries = entries.filter((entry) =>
    entry.startsWith("extension/server/")
  );
  assert.deepEqual(
    serverEntries,
    [serverEntry],
    "VSIX contains only the current host server"
  );
  const cliEntries = entries.filter((entry) =>
    entry.startsWith("extension/cli/")
  );
  assert.deepEqual(
    cliEntries,
    [cliEntry],
    "VSIX contains only the current host CLI"
  );
  const packagedStandardLibrary = entries.filter((entry) =>
    entry.startsWith("extension/standard-library/")
  );
  assert.deepEqual(
    packagedStandardLibrary.sort(),
    standardLibraryEntries.sort(),
    "VSIX contains exactly the staged standard-library manifest and sources"
  );
  assert.ok(
    packagedStandardLibrary.every(
      (entry) => entry.endsWith(".fpas") || entry.endsWith("stdlib.fpasprj")
    ),
    "VSIX excludes standard-library sidecars and unrelated files"
  );

  const buildRoots = [
    extensionRoot,
    path.resolve(extensionRoot, "..", "..")
  ].flatMap((root) => [root, root.replaceAll("\\", "/")]);
  for (const entry of entries.filter((name) =>
    /\.(?:fpas|fpasprj|json|js|md|xml)$/iu.test(name)
  )) {
    const content = archive.readAsText(entry);
    for (const root of buildRoots) {
      assert.ok(
        !content.includes(root),
        `VSIX text file excludes the local build path: ${entry}`
      );
    }
    assert.ok(
      !/(?:[a-zA-Z]:[\\/](?:Users|projects)[\\/]|\/(?:home|Users)\/[^/\s"']+)/u.test(
        content
      ),
      `VSIX text file excludes machine-specific paths: ${entry}`
    );
  }
}

function collectFiles(root, relativeDirectory = "") {
  const files = [];
  const directory = path.join(root, relativeDirectory);
  const entries = readdirSync(directory, { withFileTypes: true });
  entries.sort((left, right) => left.name.localeCompare(right.name));
  for (const entry of entries) {
    const relativePath = path.join(relativeDirectory, entry.name);
    if (entry.isDirectory()) {
      files.push(...collectFiles(root, relativePath));
    } else if (entry.isFile()) {
      files.push(relativePath);
    }
  }
  return files;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const vsixPath = process.argv[2]
    ? path.resolve(process.argv[2])
    : defaultVsixPath;
  verifyPackage(vsixPath);
  console.log(`Package verification passed: ${vsixPath}`);
}
