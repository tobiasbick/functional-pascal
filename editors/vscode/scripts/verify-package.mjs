import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import AdmZip from "adm-zip";

import {
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
  const serverEntry =
    `extension/server/${hostTarget}/${executableName}`;
  const archive = new AdmZip(vsixPath);
  const entries = archive.getEntries().map((entry) => entry.entryName);
  const entrySet = new Set(entries);
  const expectedEntries = [
    "[Content_Types].xml",
    "extension.vsixmanifest",
    "extension/LICENSE.txt",
    "extension/language-configuration.json",
    "extension/out/src/extension.js",
    "extension/package.json",
    "extension/readme.md",
    serverEntry,
    "extension/syntaxes/fpas.tmLanguage.json"
  ].sort();
  assert.deepEqual(
    [...entries].sort(),
    expectedEntries,
    "VSIX contains exactly the intended runtime files"
  );

  for (const required of [
    "extension/package.json",
    "extension/out/src/extension.js",
    "extension/readme.md",
    "extension/LICENSE.txt",
    "extension/language-configuration.json",
    "extension/syntaxes/fpas.tmLanguage.json",
    serverEntry
  ]) {
    assert.ok(entrySet.has(required), `VSIX contains ${required}`);
  }
  assert.ok(
    archive.readFile(serverEntry)?.length > 0,
    "packaged language server is non-empty"
  );
  if (executableName === "fpas-lsp") {
    const serverMode = archive.getEntry(serverEntry).attr >>> 16;
    assert.ok(
      (serverMode & 0o111) !== 0,
      "packaged Unix language server is executable"
    );
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

  const packagedLanguageConfiguration = JSON.parse(
    archive.readAsText("extension/language-configuration.json")
  );
  assert.equal(packagedLanguageConfiguration.comments.lineComment, "//");

  const packagedGrammar = JSON.parse(
    archive.readAsText("extension/syntaxes/fpas.tmLanguage.json")
  );
  assert.equal(packagedGrammar.scopeName, "source.fpas");

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
    compiledExtension.includes("Content-Length"),
    "compiled extension bundles the stdio language client"
  );
  assert.ok(
    compiledExtension.includes("server") &&
      compiledExtension.includes("fpas-lsp"),
    "compiled extension resolves the packaged language server"
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

  const buildRoots = [
    extensionRoot,
    path.resolve(extensionRoot, "..", "..")
  ].flatMap((root) => [root, root.replaceAll("\\", "/")]);
  for (const entry of entries.filter((name) =>
    /\.(?:json|js|md|xml)$/iu.test(name)
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

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const vsixPath = process.argv[2]
    ? path.resolve(process.argv[2])
    : defaultVsixPath;
  verifyPackage(vsixPath);
  console.log(`Package verification passed: ${vsixPath}`);
}
