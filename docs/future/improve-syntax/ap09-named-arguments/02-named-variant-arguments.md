# AP09.2: Named arguments for enum variant constructors

Package: [AP09: Named arguments](README.md)

Status: complete.

## Result

Current non-generic enum variants accept positional or fully named arguments
using declared payload-field names, including short, qualified and imported
constructors. Mapping is case-insensitive; written evaluation order is retained.
Patterns remain positional. `Ok`, `Error`, and `Some` have no declared field
names and accept positional arguments only (FP3026).

Variant fields are document symbols. Navigation, signature help and rename
cover named labels. Registry-only keyword-named variants lack navigable field
symbols. Named construction of generic enums belongs to AP24.3. AP10.1 covers
typed record labels; AP10.3 removes anonymous record literals, whose labels
remain outside editor field rename.

## Regression coverage

Tests cover constructor forms, compiled-unit imports, traces, unknown/duplicate/
missing/mixed names, built-in constructors, positional patterns and editor labels.
See [enums](../../../pascal/language/types/enums.md).
