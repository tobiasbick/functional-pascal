# AP42.1: Collection contract

Package: [AP42](README.md). Status: Decision pending; decision work package.

## Scope and deliverable

Record the user's choices for indexing/membership aliases, repetition,
Option-valued index search, and sortability. Audit the native catalog and its
contract allowlist before removing any operation. Decide exact result types,
validation failures, and generic constraints. Retain meaningful differences
between checked lookup and direct indexing, and between copying and mutation.

## Prerequisites and affected areas

AP06 is complete. Consult AP33/AP37 decisions if the selected design depends
on numeric ordering or mutation; do not silently take ownership of them.
Change planning files only. Current native array/string/dictionary references
remain the implementation baseline.

## Verification

The decision includes effectful `MakeValue() in MakeArray()` versus
`MakeArray().Contains(MakeValue())`: these evaluate in a different order and
cannot be blindly rewritten. Cover Unicode, invalid FromChar input, missing
indexes, generic arrays, and empty/nonempty unsupported Sort element types.
