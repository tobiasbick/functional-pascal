import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

import vscodeOniguruma from "vscode-oniguruma";
import vscodeTextmate from "vscode-textmate";

const { OnigScanner, OnigString, loadWASM } = vscodeOniguruma;
const { INITIAL, Registry, parseRawGrammar } = vscodeTextmate;

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const extensionRoot = path.resolve(scriptDirectory, "..");
const grammarPath = path.join(
  extensionRoot,
  "syntaxes",
  "fpas.tmLanguage.json"
);
const fixtureDirectory = path.join(extensionRoot, "test", "grammar");

async function createGrammar() {
  const wasmPath = path.join(
    extensionRoot,
    "node_modules",
    "vscode-oniguruma",
    "release",
    "onig.wasm"
  );
  const wasm = await readFile(wasmPath);
  await loadWASM(
    wasm.buffer.slice(wasm.byteOffset, wasm.byteOffset + wasm.byteLength)
  );

  const grammarSource = await readFile(grammarPath, "utf8");
  const registry = new Registry({
    onigLib: Promise.resolve({
      createOnigScanner: (patterns) => new OnigScanner(patterns),
      createOnigString: (source) => new OnigString(source)
    }),
    loadGrammar: async (scopeName) => {
      if (scopeName !== "source.fpas") {
        return null;
      }
      return parseRawGrammar(grammarSource, grammarPath);
    }
  });

  const grammar = await registry.loadGrammar("source.fpas");
  assert.ok(grammar, "source.fpas grammar loads");
  return grammar;
}

async function tokenizeFixture(grammar, fixtureName) {
  const source = await readFile(
    path.join(fixtureDirectory, fixtureName),
    "utf8"
  );
  const lines = source.split(/\r?\n/u);
  const tokensByLine = [];
  let ruleStack = INITIAL;

  for (const line of lines) {
    const result = grammar.tokenizeLine(line, ruleStack);
    ruleStack = result.ruleStack;
    tokensByLine.push(
      result.tokens.map((token, index) => ({
        startIndex: token.startIndex,
        endIndex: result.tokens[index + 1]?.startIndex ?? line.length,
        scopes: token.scopes
      }))
    );
  }

  return { lines, tokensByLine };
}

function tokenAt(fixture, lineFragment, text, occurrence = 0) {
  const lineIndex = fixture.lines.findIndex((line) =>
    line.includes(lineFragment)
  );
  assert.notEqual(lineIndex, -1, `fixture contains line ${lineFragment}`);

  const line = fixture.lines[lineIndex];
  let textIndex = -1;
  let searchFrom = 0;
  for (let current = 0; current <= occurrence; current += 1) {
    textIndex = line.indexOf(text, searchFrom);
    assert.notEqual(textIndex, -1, `${lineFragment} contains ${text}`);
    searchFrom = textIndex + text.length;
  }

  const token = fixture.tokensByLine[lineIndex].find(
    (candidate) =>
      candidate.startIndex <= textIndex && candidate.endIndex > textIndex
  );
  assert.ok(token, `token exists at ${lineFragment}: ${text}`);
  return token;
}

function assertScope(token, expectedScope) {
  assert.ok(
    token.scopes.includes(expectedScope),
    `expected ${expectedScope}, got ${token.scopes.join(", ")}`
  );
}

function assertNoKeywordScope(token) {
  assert.ok(
    !token.scopes.some((scope) => scope.startsWith("keyword.")),
    `expected no keyword scope, got ${token.scopes.join(", ")}`
  );
}

async function verifyPositiveScopes(grammar) {
  const fixture = await tokenizeFixture(grammar, "positive.fpas");

  assertScope(
    tokenAt(fixture, "program HighlightingShowcase", "program"),
    "keyword.declaration.module.fpas"
  );
  assertScope(
    tokenAt(
      fixture,
      "program HighlightingShowcase",
      "HighlightingShowcase"
    ),
    "entity.name.namespace.fpas"
  );
  assertScope(
    tokenAt(fixture, "Point = record", "Point"),
    "entity.name.type.fpas"
  );
  assertScope(
    tokenAt(fixture, "function Distance", "Distance"),
    "entity.name.function.fpas"
  );
  assertScope(
    tokenAt(fixture, "if Mask >= 1", "if"),
    "keyword.control.fpas"
  );
  assertScope(
    tokenAt(fixture, "X: integer", "integer"),
    "support.type.builtin.fpas"
  );
  assertScope(
    tokenAt(fixture, "X := 42", "42"),
    "constant.numeric.integer.fpas"
  );
  assertScope(
    tokenAt(fixture, "Y := 3.5", "3.5"),
    "constant.numeric.real.fpas"
  );
  assertScope(
    tokenAt(fixture, "It''s the origin", "It"),
    "string.quoted.single.fpas"
  );
  assertScope(
    tokenAt(fixture, "It''s the origin", "''"),
    "constant.character.escape.apostrophe.fpas"
  );
  assertScope(
    tokenAt(fixture, "Positive syntax-highlighting fixture", "Positive"),
    "comment.line.double-slash.fpas"
  );
  assertScope(
    tokenAt(fixture, "Point := Point", ":="),
    "keyword.operator.fpas"
  );
  assertScope(
    tokenAt(fixture, "if Mask >= 1", ">="),
    "keyword.operator.fpas"
  );
  assertScope(
    tokenAt(fixture, "return Value.X + Value.Y;", "+"),
    "keyword.operator.fpas"
  );
  assertScope(
    tokenAt(fixture, "uses Std.Console", "Std.Console"),
    "entity.name.namespace.fpas"
  );
}

async function verifyNegativeScopes(grammar) {
  const fixture = await tokenizeFixture(grammar, "negative.fpas");

  assertNoKeywordScope(
    tokenAt(fixture, "beginValue: string", "beginValue")
  );
  assertNoKeywordScope(tokenAt(fixture, "gifted: boolean", "gifted"));
  assertNoKeywordScope(tokenAt(fixture, "endif: integer", "endif"));
  assertNoKeywordScope(tokenAt(fixture, "const shl", "shl"));
  assertNoKeywordScope(tokenAt(fixture, "const SHR", "SHR"));
  const constantName = tokenAt(
    fixture,
    "RecordCount: integer := 1",
    "RecordCount"
  );
  assertScope(constantName, "entity.name.constant.fpas");
  assert.ok(
    !constantName.scopes.includes("entity.name.type.fpas"),
    "constant declarations are not classified as type declarations"
  );

  const stringKeyword = tokenAt(
    fixture,
    "'if then begin end'",
    "if"
  );
  assertScope(stringKeyword, "string.quoted.single.fpas");
  assertNoKeywordScope(stringKeyword);

  const lineCommentKeyword = tokenAt(
    fixture,
    "// function return record",
    "function"
  );
  assertScope(lineCommentKeyword, "comment.line.double-slash.fpas");
  assertNoKeywordScope(lineCommentKeyword);

  const secondLineCommentKeyword = tokenAt(
    fixture,
    "// while repeat until",
    "while"
  );
  assertScope(secondLineCommentKeyword, "comment.line.double-slash.fpas");
  assertNoKeywordScope(secondLineCommentKeyword);
}

async function verifyEdgeScopes(grammar) {
  const fixture = await tokenizeFixture(grammar, "edge.fpas");

  assertScope(
    tokenAt(fixture, "It''s Grüße aus 東京", "''"),
    "constant.character.escape.apostrophe.fpas"
  );
  assertScope(
    tokenAt(fixture, "It''s Grüße aus 東京", "東京"),
    "string.quoted.single.fpas"
  );
  assertScope(
    tokenAt(fixture, "HexValue: integer := $2A", "$2A"),
    "constant.numeric.hex.fpas"
  );
  assertScope(
    tokenAt(fixture, "// outer { inner }", "AfterBrace"),
    "comment.line.double-slash.fpas"
  );
  assertScope(
    tokenAt(fixture, "// outer (* inner *)", "AfterParen"),
    "comment.line.double-slash.fpas"
  );
  assertScope(
    tokenAt(fixture, "Text := 'unfinished", "unfinished"),
    "string.quoted.single.fpas"
  );
}

async function verifyReservedKeywordScopes(grammar) {
  const fixture = await tokenizeFixture(grammar, "reserved_keywords.fpas");
  for (const spellings of [
    ["elsif", "ELSIF", "ElSiF"],
    ["when", "WHEN", "WhEn"],
    ["null", "NULL", "NuLl"],
    ["discard", "DISCARD", "DiScArD"]
  ]) {
    for (const spelling of spellings) {
      assertScope(
        tokenAt(fixture, spellings.join(" "), spelling),
        "keyword.control.fpas"
      );
    }
  }
  for (const identifier of ["NullValue", "WhenValue", "ElsifValue"]) {
    assertNoKeywordScope(tokenAt(fixture, `const ${identifier}`, identifier));
  }
  for (const keyword of ["elsif", "WHEN", "NuLl"]) {
    const token = tokenAt(fixture, "'elsif WHEN NuLl'", keyword);
    assertScope(token, "string.quoted.single.fpas");
    assertNoKeywordScope(token);
  }
  for (const keyword of ["ELSIF", "WHEN", "NULL"]) {
    const token = tokenAt(fixture, "// ELSIF WHEN NULL", keyword);
    assertScope(token, "comment.line.double-slash.fpas");
    assertNoKeywordScope(token);
  }
  const jsonNull = tokenAt(fixture, "'{\"value\":null}'", "null");
  assertScope(jsonNull, "string.quoted.single.fpas");
  assertNoKeywordScope(jsonNull);
}

async function verifyDeclarationClosers(grammar) {
  const fixture = await tokenizeFixture(grammar, "declaration_closers.fpas");
  const configuration = JSON.parse(await readFile(
    path.join(extensionRoot, "language-configuration.json"), "utf8"
  ));
  const increase = new RegExp(configuration.indentationRules.increaseIndentPattern);
  const decrease = new RegExp(configuration.indentationRules.decreaseIndentPattern);
  for (const kind of ["function", "procedure", "record", "enum", "unit"]) {
    const line = `end ${kind};`;
    assertScope(tokenAt(fixture, line, "end"), "keyword.control.fpas");
    assertScope(tokenAt(fixture, line, kind),
      kind === "record" || kind === "enum"
        ? "storage.type.composite.fpas" : "keyword.declaration.fpas");
    for (const spelling of [line, line.toUpperCase(), line.slice(0, -1)]) {
      assert.ok(decrease.test(spelling), `${spelling} decreases indentation`);
      assert.ok(!increase.test(spelling), `${spelling} does not open another block`);
    }
  }
  const snippets = JSON.parse(await readFile(
    path.join(extensionRoot, "snippets", "fpas.json"), "utf8"
  ));
  for (const [name, kind] of [["Function declaration", "function"],
    ["Procedure declaration", "procedure"], ["Record type", "record"], ["Unit", "unit"]]) {
    assert.equal(snippets[name].body.at(-1), `end ${kind};`);
  }
}

async function verifyControlBlocks(grammar) {
  const fixture = await tokenizeFixture(grammar, "control_blocks.fpas");
  const configuration = JSON.parse(await readFile(
    path.join(extensionRoot, "language-configuration.json"), "utf8"
  ));
  const increase = new RegExp(configuration.indentationRules.increaseIndentPattern);
  const decrease = new RegExp(configuration.indentationRules.decreaseIndentPattern);
  for (const line of ["if true then", "elsif false then", "else",
    "for I: integer := 1 to 2 do", "while false do", "repeat"]) {
    assert.ok(increase.test(line), `${line} opens a body`);
    assert.ok(increase.test(`${line.toUpperCase()} // body`));
  }
  for (const kind of ["if", "for", "while"]) {
    const line = `end ${kind};`;
    assertScope(tokenAt(fixture, line, "end"), "keyword.control.fpas");
    assertScope(tokenAt(fixture, line, kind), "keyword.control.fpas");
    assert.ok(decrease.test(line));
    assert.ok(!increase.test(line));
    assert.ok(!increase.test(line.toUpperCase()));
  }
  for (const line of ["elsif false then", "else", "until true;"]) {
    assert.ok(decrease.test(line));
    assert.ok(decrease.test(line.toUpperCase()));
  }
  assertScope(tokenAt(fixture, "elsif false then", "elsif"), "keyword.control.fpas");
  assertScope(tokenAt(fixture, "null;", "null"), "keyword.control.fpas");
  const snippets = JSON.parse(await readFile(
    path.join(extensionRoot, "snippets", "fpas.json"), "utf8"
  ));
  for (const [name, kind] of [["If statement", "if"], ["For loop", "for"],
    ["While loop", "while"]]) {
    assert.equal(snippets[name].body.at(-1), `end ${kind};`);
    assert.ok(!snippets[name].body.includes("begin"));
    assert.ok(snippets[name].body.some((line) => line.includes("null;")));
  }
  assert.ok(snippets["Repeat loop"].body.some((line) => line.includes("null;")));
}

async function verifyCaseBlocks(grammar) {
  const fixture = await tokenizeFixture(grammar, "case_blocks.fpas");
  const configuration = JSON.parse(await readFile(
    path.join(extensionRoot, "language-configuration.json"), "utf8"
  ));
  const increase = new RegExp(configuration.indentationRules.increaseIndentPattern);
  const decrease = new RegExp(configuration.indentationRules.decreaseIndentPattern);
  for (const line of ["case Some(1) of", "when Some(const Value) if Value > 0:", "when 0, 1:"]) {
    assert.ok(increase.test(line), `${line} opens a body`);
    assert.ok(increase.test(`${line.toUpperCase()} // body`));
  }
  for (const line of ["when Some(const Value):", "when None:", "else", "end case;"]) {
    assert.ok(decrease.test(line), `${line} decreases indentation`);
    assert.ok(decrease.test(line.toUpperCase()));
  }
  assert.ok(!increase.test("end case;"));
  assert.ok(!increase.test("END CASE; // ending"));
  assertScope(tokenAt(fixture, "when Some(const Value) if Value > 0:", "when"), "keyword.control.fpas");
  assertScope(tokenAt(fixture, "when Some(const Value) if Value > 0:", "const"), "storage.type.constant.fpas");
  assertScope(tokenAt(fixture, "end case;", "end"), "keyword.control.fpas");
  assertScope(tokenAt(fixture, "end case;", "case"), "keyword.control.fpas");
  const snippets = JSON.parse(await readFile(
    path.join(extensionRoot, "snippets", "fpas.json"), "utf8"
  ));
  const body = snippets["Case statement"].body;
  assert.equal(body.at(-1), "end case;");
  assert.ok(body[1].trimStart().startsWith("when "));
  assert.ok(body.some((line) => line.includes("null;")));
  assert.ok(!body.includes("begin"));
}

async function verifyExpressionClosers(grammar) {
  const fixture = await tokenizeFixture(grammar, "expression_closers.fpas");
  const configuration = JSON.parse(await readFile(
    path.join(extensionRoot, "language-configuration.json"), "utf8"
  ));
  const increase = new RegExp(configuration.indentationRules.increaseIndentPattern);
  const decrease = new RegExp(configuration.indentationRules.decreaseIndentPattern);
  for (const kind of ["function", "procedure", "with"]) {
    const line = `end ${kind}`;
    assertScope(tokenAt(fixture, line, "end"), "keyword.control.fpas");
    assertScope(tokenAt(fixture, line, kind), kind === "with"
      ? "keyword.control.fpas" : "keyword.declaration.fpas");
    for (const spelling of [line, `${line}, Other`, `${line});`, line.toUpperCase()]) {
      assert.ok(decrease.test(spelling), `${spelling} decreases indentation`);
      assert.ok(!increase.test(spelling), `${spelling} does not open a body`);
    }
  }
  assert.ok(increase.test("const Moved: Point := Original with"));
  assert.ok(increase.test("ORIGINAL WITH // overrides"));
  const snippets = JSON.parse(await readFile(
    path.join(extensionRoot, "snippets", "fpas.json"), "utf8"
  ));
  for (const [name, kind] of [["Anonymous function", "function"],
    ["Anonymous procedure", "procedure"], ["Record update", "with"]]) {
    assert.equal(snippets[name].body.at(-1), `end ${kind}`);
  }
}

/** Loads the grammar and verifies positive, negative, and edge-case scopes. */
async function verifyIndividualDeclarations(grammar) {
  const fixture = await tokenizeFixture(grammar, "individual_declarations.fpas");
  for (const name of ["First", "Second"]) {
    const line = `public type ${name}`;
    assertScope(tokenAt(fixture, line, "public"), "keyword.declaration.visibility.fpas");
    assertScope(tokenAt(fixture, line, "type"), "keyword.declaration.type.fpas");
    assertScope(tokenAt(fixture, line, name), "entity.name.type.fpas");
  }
  for (const name of ["A", "B", "C", "D"]) {
    const line = `public const ${name}`;
    assertScope(tokenAt(fixture, line, "const"), "storage.type.constant.fpas");
    assertScope(tokenAt(fixture, line, name), "entity.name.constant.fpas");
  }
  for (const [prefix, names] of [["var", ["E", "F"]]]) {
    for (const name of names) {
      const line = `public ${prefix} ${name}`;
      assertScope(tokenAt(fixture, line, "var"), "storage.type.fpas");
      assertScope(tokenAt(fixture, line, name), "variable.other.definition.fpas");
    }
  }
  assertNoKeywordScope(tokenAt(fixture, "public Value: First", "Value"));
  assertNoKeywordScope(tokenAt(fixture, "public var mutable", "mutable"));
}

async function verifyImportAliases(grammar) {
  const fixture = await tokenizeFixture(grammar, "import_aliases.fpas");
  for (const [line, modifier] of [["uses Demo.Math as M", "as"],
    ["As as First", "as"], ["Demo.As as As", "as"],
    ["AS // contextual keyword", "AS"]]) {
    assertScope(tokenAt(fixture, line, modifier), "keyword.declaration.import.fpas");
  }
  for (const [line, name, occurrence] of [["uses Demo.Math as M", "M", 1],
    ["As as First", "As", 0], ["Demo.As as As", "As", 0],
    ["Demo.As as As", "As", 1], ["       As", "As", 0],
    ["       Other;", "Other", 0]]) {
    assertScope(tokenAt(fixture, line, name, occurrence), "entity.name.namespace.fpas");
    assertNoKeywordScope(tokenAt(fixture, line, name, occurrence));
  }
  for (const line of ["const As:", "const Value:", "discard As.Answer()"])
    assertNoKeywordScope(tokenAt(fixture, line, "As"));
  assertScope(tokenAt(fixture, "// uses", "as"), "comment.line.double-slash.fpas");
  assertScope(tokenAt(fixture, "const Text:", "as"), "string.quoted.single.fpas");
}

export async function verifyGrammar() {
  const grammar = await createGrammar();
  await verifyDeclarationClosers(grammar);
  await verifyControlBlocks(grammar);
  await verifyCaseBlocks(grammar);
  await verifyExpressionClosers(grammar);
  await verifyPositiveScopes(grammar);
  await verifyIndividualDeclarations(grammar);
  await verifyImportAliases(grammar);
  await verifyNegativeScopes(grammar);
  await verifyEdgeScopes(grammar);
  await verifyReservedKeywordScopes(grammar);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  await verifyGrammar();
  console.log("Grammar verification passed.");
}
