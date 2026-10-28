# Text and parsing

Strings, conversions, binary-text encodings, structured parsing, JSON, and TOML.

String operations are native and need no `uses` clause. The `Std.*` units below
require imports.

| Topic | Description |
|-------|-------------|
| [`String operations`](../../language/types/string/README.md) | String operations |
| [`Std.Text.Format`](format.md) | Locale-independent text formatting |
| [`Std.Encoding`](encoding.md) | Strict Base64, unpadded URL-safe Base64, and hex byte codecs |
| [`Std.Text.Parse`](parse.md) | Result-returning `Int`, `Real`, `Bool`, `HexInt` |
| [`Std.Text.Utf8`](../network/utf8.md) | UTF-8 encoding and validated decoding |
| [`Std.Encoding.Json`](json.md) | JSON trees |
| [`Std.Encoding.Json.Fields`](json-fields.md) | Typed JSON object field access |
| [`Std.Encoding.Toml`](toml.md) | TOML trees |
| [`Std.Encoding.Toml.Fields`](toml-fields.md) | Typed TOML table key access |

## See also

- [Standard library index](../README.md)
- [Collections](../collections/README.md)
