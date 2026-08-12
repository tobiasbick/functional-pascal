import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const extensionRoot = path.resolve(scriptDirectory, "..");

/** Validates the local extension manifest and declarative language files. */
export async function verifyManifest() {
  const manifestPath = path.join(extensionRoot, "package.json");
  const manifest = JSON.parse(await readFile(manifestPath, "utf8"));

  assert.equal(
    `${manifest.publisher}.${manifest.name}`,
    "functional-pascal.functional-pascal"
  );
  assert.equal(manifest.version, "0.3.0");
  assert.equal(manifest.main, "./out/src/extension.js");
  assert.equal(manifest.engines?.vscode, "^1.91.0");
  assert.equal(manifest.scripts?.package, "node scripts/package.mjs");
  assert.equal(manifest.scripts?.publish, undefined);

  const commands = manifest.contributes?.commands ?? [];
  assert.deepEqual(
    commands.map((value) => value.command),
    [
      "functionalPascal.showOutput",
      "functionalPascal.restartLanguageServer",
      "functionalPascal.selectProject",
      "functionalPascal.checkProject",
      "functionalPascal.buildProject",
      "functionalPascal.runProject",
      "functionalPascal.testProject",
      "functionalPascal.formatProject",
      "functionalPascal.checkProjectFormatting",
      "functionalPascal.cancelOperation",
      "functionalPascal.refreshTests",
      "functionalPascal.debug.insertDictionaryEntry",
      "functionalPascal.debug.removeDictionaryEntry",
      "functionalPascal.debug.replaceDictionaryKey"
    ]
  );
  assert.ok(
    commands.every((value) => value.category === "Functional Pascal")
  );
  assert.deepEqual(
    manifest.contributes?.configuration?.properties?.[
      "functionalPascal.testTimeoutSeconds"
    ],
    {
      type: "number",
      minimum: 1,
      default: 10,
      description: "Timeout in seconds for each test started from the Testing view."
    }
  );

  assert.equal(manifest.activationEvents, undefined);
  assert.deepEqual(manifest.extensionKind, ["ui"]);
  assert.equal(manifest.dependencies?.["vscode-languageclient"], "10.1.0");

  assert.deepEqual(manifest.contributes?.languages, [
    {
      id: "fpas",
      aliases: ["Functional Pascal", "fpas"],
      extensions: [".fpas"],
      configuration: "./language-configuration.json"
    }
  ]);
  assert.deepEqual(manifest.contributes?.grammars, [
    {
      language: "fpas",
      scopeName: "source.fpas",
      path: "./syntaxes/fpas.tmLanguage.json"
    }
  ]);
  assert.deepEqual(manifest.contributes?.snippets, [
    {
      language: "fpas",
      path: "./snippets/fpas.json"
    }
  ]);
  assert.deepEqual(manifest.contributes?.breakpoints, [
    {
      language: "fpas"
    }
  ]);
  assert.deepEqual(manifest.contributes?.debuggers, [
    {
      type: "fpas",
      label: "Functional Pascal",
      languages: ["fpas"],
      configurationAttributes: {
        launch: {
          required: ["program"],
          properties: {
            program: {
              type: "string",
              description: "FPAS source, program project, workspace, or compiled image."
            },
            cwd: {
              type: "string",
              description: "Debugger working directory."
            },
            args: {
              type: "array",
              items: { type: "string" },
              default: []
            },
            stopOnEntry: { type: "boolean", default: false },
            sourceRoot: {
              type: "string",
              description: "Required source root for .fpascp targets."
            }
          }
        }
      },
      initialConfigurations: [
        {
          type: "fpas",
          request: "launch",
          name: "Debug Functional Pascal",
          program: "${file}",
          cwd: "${workspaceFolder}",
          stopOnEntry: false
        }
      ]
    }
  ]);

  const snippets = JSON.parse(
    await readFile(path.join(extensionRoot, "snippets", "fpas.json"), "utf8")
  );
  assert.ok(Object.keys(snippets).length >= 10);

  const languageConfiguration = JSON.parse(
    await readFile(
      path.join(extensionRoot, "language-configuration.json"),
      "utf8"
    )
  );
  assert.equal(languageConfiguration.comments.lineComment, "//");
  assert.equal(languageConfiguration.comments.blockComment, undefined);
  assert.ok(
    !languageConfiguration.autoClosingPairs.some(
      (pair) => pair.open === "{" || pair.open === "(*"
    )
  );
  assert.ok(
    !languageConfiguration.surroundingPairs.some(
      (pair) => pair[0] === "{" || pair[0] === "(*"
    )
  );
  assert.ok(languageConfiguration.brackets.length > 0);
  assert.ok(languageConfiguration.autoClosingPairs.length > 0);
  assert.ok(languageConfiguration.surroundingPairs.length > 0);
  assert.ok(languageConfiguration.indentationRules);
  assert.ok(languageConfiguration.folding?.markers);

  const grammar = JSON.parse(
    await readFile(
      path.join(extensionRoot, "syntaxes", "fpas.tmLanguage.json"),
      "utf8"
    )
  );
  assert.equal(grammar.scopeName, "source.fpas");
  assert.ok(grammar.patterns.length > 0);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  await verifyManifest();
  console.log("Manifest verification passed.");
}
