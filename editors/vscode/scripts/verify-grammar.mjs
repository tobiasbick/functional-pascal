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
    "comment.line.documentation.fpas"
  );
  assertScope(
    tokenAt(fixture, "{ Brace comment. }", "Brace"),
    "comment.block.brace.fpas"
  );
  assertScope(
    tokenAt(fixture, "(* Parenthesized comment. *)", "Parenthesized"),
    "comment.block.parenthesized.fpas"
  );
  assertScope(
    tokenAt(fixture, "Point := record", ":="),
    "keyword.operator.fpas"
  );
  assertScope(
    tokenAt(fixture, "if Mask >= 1", ">="),
    "keyword.operator.fpas"
  );
  assertScope(
    tokenAt(fixture, "return Value.X + Value.Y", "+"),
    "keyword.operator.fpas"
  );
  assertScope(
    tokenAt(fixture, "uses Std.Console", "Std.Console"),
    "variable.other.qualified.fpas"
  );
}

async function verifyNegativeScopes(grammar) {
  const fixture = await tokenizeFixture(grammar, "negative.fpas");

  assertNoKeywordScope(
    tokenAt(fixture, "beginValue: string", "beginValue")
  );
  assertNoKeywordScope(tokenAt(fixture, "gifted: boolean", "gifted"));
  assertNoKeywordScope(tokenAt(fixture, "endif: integer", "endif"));
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

  const blockCommentKeyword = tokenAt(
    fixture,
    "{ while repeat until }",
    "while"
  );
  assertScope(blockCommentKeyword, "comment.block.brace.fpas");
  assertNoKeywordScope(blockCommentKeyword);
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
    tokenAt(fixture, "{ outer { inner }", "inner"),
    "comment.block.brace.fpas"
  );
  assert.ok(
    !tokenAt(fixture, "{ outer { inner }", "AfterBrace").scopes.includes(
      "comment.block.brace.fpas"
    ),
    "the first closing brace ends a non-nesting brace comment"
  );
  assertScope(
    tokenAt(fixture, "(* outer (* inner *)", "inner"),
    "comment.block.parenthesized.fpas"
  );
  assert.ok(
    !tokenAt(fixture, "(* outer (* inner *)", "AfterParen").scopes.includes(
      "comment.block.parenthesized.fpas"
    ),
    "the first closing delimiter ends a non-nesting parenthesized comment"
  );
  assertScope(
    tokenAt(fixture, "Text := 'unfinished", "unfinished"),
    "string.quoted.single.fpas"
  );
}

/** Loads the grammar and verifies positive, negative, and edge-case scopes. */
export async function verifyGrammar() {
  const grammar = await createGrammar();
  await verifyPositiveScopes(grammar);
  await verifyNegativeScopes(grammar);
  await verifyEdgeScopes(grammar);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  await verifyGrammar();
  console.log("Grammar verification passed.");
}
