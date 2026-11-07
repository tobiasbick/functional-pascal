import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';

const require = createRequire(import.meta.url);
const { Registry, parseRawGrammar } = require('vscode-textmate');
const oniguruma = require('vscode-oniguruma');
const wasm = await readFile(require.resolve('vscode-oniguruma/release/onig.wasm'));
await oniguruma.loadWASM(wasm.buffer.slice(wasm.byteOffset, wasm.byteOffset + wasm.byteLength));
const grammarFile = fileURLToPath(new URL('../syntaxes/fpas.tmLanguage.json', import.meta.url));
const registry = new Registry({
  onigLib: Promise.resolve({
    createOnigScanner: patterns => new oniguruma.OnigScanner(patterns),
    createOnigString: text => new oniguruma.OnigString(text),
  }),
  loadGrammar: async () => parseRawGrammar(await readFile(grammarFile, 'utf8'), grammarFile),
});
const grammar = await registry.loadGrammar('source.fpas');
for (const [line, token, expected] of [
  ['program Demo;', 'program', 'keyword.other.fpas'],
  ['Program Demo;', 'Program', 'source.fpas'],
  ['const @end: int := #65;', '@end', 'variable.other.escaped.fpas'],
  ['const @end: int := #65;', '#65', 'constant.character.fpas'],
  ['const Hex: uint := $FFu;', '$FFu', 'constant.numeric.fpas'],
  ['const Number: decimal := 1_000.5M;', '1_000.5M', 'constant.numeric.fpas'],
  ["'It''s // a string' // comment", '// a string', 'string.quoted.single.fpas'],
  ["'It''s // a string' // comment", '// comment', 'comment.line.double-slash.fpas'],
  ['end program;', 'end', 'keyword.control.fpas'],
  ['(* ordinary tokens *)', 'ordinary', 'source.fpas'],
]) {
  const start = line.indexOf(token);
  const scopes = grammar.tokenizeLine(line).tokens.find(entry => entry.startIndex <= start && entry.endIndex > start)?.scopes;
  assert.equal(scopes?.at(-1), expected, `${line} at ${token}`);
}
const broken = grammar.tokenizeLine("'incomplete");
const next = grammar.tokenizeLine('end program;', broken.ruleStack);
assert.equal(next.tokens[0].scopes.at(-1), 'keyword.control.fpas');
console.log('TextMate tokenization passed for CLR syntax and incomplete strings.');
