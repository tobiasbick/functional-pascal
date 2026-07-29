import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath } from "node:url";

import AdmZip from "adm-zip";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const extensionRoot = path.resolve(scriptDirectory, "..");
const defaultVsixPath = path.join(
  extensionRoot,
  "dist",
  "functional-pascal-0.0.1-bootstrap.vsix"
);

/** Verifies that the local VSIX contains only intended runtime files. */
export function verifyPackage(vsixPath = defaultVsixPath) {
  const archive = new AdmZip(vsixPath);
  const entries = archive.getEntries().map((entry) => entry.entryName);
  const entrySet = new Set(entries);

  for (const required of [
    "extension/package.json",
    "extension/out/src/extension.js",
    "extension/readme.md",
    "extension/LICENSE.txt",
    "extension/language-configuration.json",
    "extension/syntaxes/fpas.tmLanguage.json"
  ]) {
    assert.ok(entrySet.has(required), `VSIX contains ${required}`);
  }

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
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const vsixPath = process.argv[2]
    ? path.resolve(process.argv[2])
    : defaultVsixPath;
  verifyPackage(vsixPath);
  console.log(`Package verification passed: ${vsixPath}`);
}
