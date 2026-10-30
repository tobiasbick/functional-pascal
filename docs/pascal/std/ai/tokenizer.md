# `Std.Ai.Tokenizer`

Exact model-token counts and text/token-ID conversion through `tiktoken-rs`.
Vocabulary and merge data are bundled with the runtime. Operations require no
Python installation, network connection or external vocabulary file.

```pascal id=tokenizer-example run
program TokenCount;

uses Std.Ai.Tokenizer as Tokenizer, Std.Test as Test;

begin
  Test.AssertEquals(2, Tokenizer.Count('hello world', 'o200k_base').Unwrap());
  const Tokens := Tokenizer.Encode('Grüße 🙂', 'o200k_base').Unwrap();
  Test.AssertEquals('Grüße 🙂', Tokenizer.Decode(Tokens, 'o200k_base').Unwrap());
end.
```

## Quick reference

| Kind | Signature | Description |
| --- | --- | --- |
| function | `Encodings(): array of string` | Supported encoding names |
| function | `Count(Text: string; Encoding: string): result of (integer, TokenizerError)` | Exact ordinary-text token count |
| function | `Encode(Text: string; Encoding: string): result of (array of integer, TokenizerError)` | Ordinary-text token IDs in order |
| function | `Decode(Tokens: array of integer; Encoding: string): result of (string, TokenizerError)` | Complete validated UTF-8 text |

Encoding names are case-sensitive: `o200k_base`, `cl100k_base`, `r50k_base`,
`p50k_base`, `p50k_edit` and `o200k_harmony`. Choose the encoding explicitly;
there is no model-name guessing or silent fallback. Token IDs are meaningful
only with the same encoding.

`Count(Text, Encoding)` equals `Encode(Text, Encoding).Unwrap().Length()` when
both succeed. Counts are BPE tokens, not bytes, Unicode scalars or words.
For example, `hello world` produces `[24912, 2375]` with `o200k_base` and
`[15339, 1917]` with `cl100k_base`.

## Ordinary text and special tokens

Count and Encode follow tiktoken's `encode_ordinary` contract. Strings such as
`<|endoftext|>` are literal text, not instructions to insert a special token.
Decode accepts known special-token IDs and reconstructs their literal spelling.
An individual token may contain only part of a UTF-8 sequence; decode the complete
token sequence together. Invalid UTF-8 is an error, without replacement characters.

These operations count supplied text only. Chat templates, message boundaries,
tool definitions and provider-specific request overhead are outside this API.

## Ownership and limits

Read-only tokenizer tables are initialized lazily and owned by one VM. Calls
reuse the tables, including concurrent tasks and callbacks. The tables are freed
with the VM's hosted state, once its session owners are released; no process-global
tokenizer singleton is retained.
First use builds the tables in memory. Tokenization is synchronous CPU work,
and ordinary/seeded debugging and recording can execute it deterministically.

Text input and decoded output are bounded to 16 MiB of UTF-8 bytes. Token arrays
and encoded results are bounded to 1,048,576 IDs. Count shares the token limit.
Failure returns a typed error and never returns a truncated result.

## Typed failures

| Variant | Meaning |
| --- | --- |
| `UnsupportedEncoding(Name: string)` | Encoding name is not supported |
| `InvalidToken(Index: integer; Value: integer)` | Token `Value` at zero-based `Index` is negative, exceeds the token-ID range or is unknown to this encoding |
| `InvalidUtf8` | Concatenated token bytes do not form valid UTF-8 |
| `LimitExceeded(Name: string; Limit: integer)` | `TextBytes`, `DecodedBytes` or `Tokens` exceeds its documented bound |
| `BackendFailure` | Bundled tokenizer initialization or execution failed |

All routines remain under the single canonical unit. The backend dependency
and encoding names do not introduce extra public units.

## Repository tools

Run `fpas run tools/agent-token-budget/agent-token-budget.fpasprj` from the
repository root to check the agent reference with `o200k_base`. File discovery,
token counting and budget validation run entirely in FPAS with the Rust-backed
standard-library API.

The optional [reference-vector generator](../../../../tools/tokenizer-reference/README.md)
uses Python tiktoken only as an independent test oracle. Normal use and regression
tests do not require it.
