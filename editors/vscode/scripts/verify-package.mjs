import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import AdmZip from "adm-zip";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const extensionRoot = path.resolve(scriptDirectory, "..");
const sourceManifest = JSON.parse(
  readFileSync(path.join(extensionRoot, "package.json"), "utf8")
);
const defaultVsixPath = path.join(
  extensionRoot,
  "dist",
  `functional-pascal-${sourceManifest.version}.vsix`
);

/** Verifies that the VSIX is platform-independent and contains no FPAS toolchain. */
export function verifyPackage(vsixPath = defaultVsixPath) {
  const archive = new AdmZip(vsixPath);
  const entries = archive.getEntries().map((entry) => entry.entryName);
  const expectedEntries = [
    "[Content_Types].xml",
    "extension.vsixmanifest",
    "extension/BUG_REPORT.md",
    "extension/LICENSE.txt",
    "extension/language-configuration.json",
    "extension/out/src/debugger/terminal/externalClient.js",
    "extension/out/src/extension.js",
    "extension/package.json",
    "extension/readme.md",
    "extension/snippets/fpas.json",
    "extension/syntaxes/fpas.tmLanguage.json"
  ].sort();
  assert.deepEqual(
    [...entries].sort(),
    expectedEntries,
    "VSIX contains exactly the intended editor runtime files"
  );

  const vsixManifest = archive.readAsText("extension.vsixmanifest");
  assert.doesNotMatch(
    vsixManifest,
    /TargetPlatform=/u,
    "VSIX is not tied to one native host target"
  );

  const packagedManifest = JSON.parse(
    archive.readAsText("extension/package.json")
  );
  assert.equal(
    packagedManifest.contributes.configuration.properties[
      "functionalPascal.executablePath"
    ].scope,
    "machine"
  );
  assert.deepEqual(packagedManifest.contributes.debuggers[0].languages, [
    "fpas",
    "fpas-project"
  ]);

  const compiledExtension = archive.readAsText("extension/out/src/extension.js");
  for (const contract of [
    "Functional Pascal extension activated.",
    "functionalPascal.selectExecutable",
    "functionalPascal.executablePath",
    "env",
    "--json",
    "lsp",
    "Content-Length"
  ]) {
    assert.ok(
      compiledExtension.includes(contract),
      `compiled extension contains ${contract}`
    );
  }

  const forbiddenPrefixes = [
    "extension/cli/",
    "extension/server/",
    "extension/standard-library/",
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
      `VSIX excludes ${entry}`
    );
    assert.ok(
      !/^[a-zA-Z]:[\\/]/u.test(entry) && !entry.startsWith("/"),
      `VSIX entry is relative: ${entry}`
    );
  }

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
