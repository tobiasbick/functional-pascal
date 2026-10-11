# Compiler and semantics review

No active findings. This file is ready for the **Spec** axis of the
[next review](README.md). Completed coverage is in
[reviewed-scope.md](reviewed-scope.md).

## Finding format

Use one heading per confirmed finding: `Snn — Pn: concrete defect`.
Each finding must contain:

1. **Status and owner:** open or repaired/verified, AP work package, affected
   crate/module and fixed reviewed revision.
2. **Requirement and reproduction:** links to the agreed AP requirement and
   implemented handbook rule, minimal valid input or negative case, public
   command/API, actual result and expected result.
3. **Cause and bounded repair:** relevant code paths, existing shared owner
   and the smallest complete correction. Call out required module splits or
   moves before implementation.
4. **Acceptance:** observable positive, negative and boundary behavior;
   distinct parser, semantic, lowering, runtime or CLI risks where applicable.
   Extend existing owning tests rather than repeating the same case everywhere.
5. **Closure evidence:** demonstrated pre-repair failure, post-repair result,
   second review, docs status and commit. Link detailed commands and results
   from [verification.md](verification.md).

Record inspected areas without confirmed findings separately and state their
limits. Do not invent new language rules to repair an implementation defect.
