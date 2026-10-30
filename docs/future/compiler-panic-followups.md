# Compiler panics and language-limit follow-ups

This document is the intake list for compiler panics and language limits discovered during normal
FPAS work. Resolved behavior belongs in the implementation, regression suite, and current
documentation; broader language proposals belong in their own future notes.

## Entry rule

When development finds a compiler panic or a language limitation that requires a workaround, add an
entry here in the same change. Include the source shape, observed failure or restriction, temporary
workaround, and a concrete later resolution with regression coverage. Do not silently extend the
language or hide the limitation inside a library implementation.

## Open entries

### Generic type aliases in facades

- **Source shape:** `public type TuiSubview of T = Subview.TuiSubview of T;`.
- **Restriction:** FP2001 permits type parameters only on record and enum
  declarations. A facade can reexport concrete types but cannot express a
  transparent parameterized alias.
- **Current implementation:** the public generic record is declared directly
  in `Std.Ui.Terminal`. Its typed cache implementation remains in a private
  source unit and returns the record's callable field. There is one public type
  and API; no second wrapper type or public subunit is introduced.
- **Later resolution:** evaluate transparent generic aliases as a separate
  language change, with explicit syntax approval. Cover generic argument
  substitution, constraints, nominal identity, visibility and methods through
  independently compiled/reexported units before migrating facade declarations.
