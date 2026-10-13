# AP09.2: Named arguments for enum variant constructors

Package: [AP09: Named arguments](README.md)

Status: complete.

## Result

Current non-generic enum variants accept positional or fully named arguments
using declared payload-field names, including short, qualified and imported
constructors. Mapping is case-insensitive; written evaluation order is retained.
[AP36](../ap36-call-and-construction-conventions/README.md) extends both complete
forms to enum/Option/Result patterns and Some/Ok/Error constructors. Their
single canonical payload label is `Value`; user enum labels remain their
declared field names. Every pattern field is explicit, with no mixing per list.

Variant fields are document symbols. Navigation, signature help and rename
cover named labels. Registry-only keyword-named variants lack navigable field
symbols. Named construction of generic enums belongs to AP24.3. AP10.1 covers
typed record labels; AP10.3 removed anonymous record literals.

## Regression coverage

Tests cover constructor forms, compiled-unit imports, traces, unknown/duplicate/
missing/mixed names, built-in constructors, positional patterns and editor labels.
See [enums](../../../pascal/language/types/enums.md).
