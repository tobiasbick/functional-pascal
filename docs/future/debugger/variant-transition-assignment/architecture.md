# Architecture

## Existing foundations

- `DebugAssignmentTarget` already preserves a root followed by ordered field
  and index selectors.
- `mutation/resolve.rs` resolves selectors only when the current runtime value
  exposes the requested active descendant.
- `calls/enum_constructor.rs` already constructs and recursively validates
  complete detached enum values.
- `mutation::commit` already atomically replaces a complete writable root or
  existing descendant and expires stopped-state handles after success.
- JSONL `expression.set`, DAP `setExpression`, and VS Code use the same
  `DebugSession` operation.

## Planned prepare flow

1. Resolve the visible writable root and every ordinary existing prefix.
2. When resolution reaches an enum, `Result`, or `Option`, interpret an exact
   variant-qualified suffix such as `Some.value`, `Ok.value`, or
   `Item.value`.
3. Resolve the target variant from portable debugger metadata and require
   exactly one payload slot.
4. Evaluate the replacement expression once under the existing detached-call,
   cancellation, depth, and operation limits.
5. Validate the replacement against that payload slot's portable type.
6. Construct a complete detached target variant.
7. Reuse the existing atomic mutation commit to replace the wrapper at its
   nearest existing writable path.
8. Expire handles and emit client invalidation only after successful commit.

## Planned ownership boundary

Create `mutation/transition/` for variant-qualified suffix resolution and
complete-value preparation. Do not add transition branches to the 475-line
`inspection/targets/payload.rs`; that module remains responsible for active
payload inspection. `mutation/resolve.rs` delegates only when an ordinary
active path cannot consume the exact qualified suffix.

## State model

No partially initialized payload is stored. The live runtime sees either the
old complete variant or the new complete variant. Evaluation, metadata lookup,
construction, and recursive validation finish before commit. A failure keeps
the old value, frame, task selection, handles, and DAP invalidation count.

Generic expired `variablesReference` values remain invalid. This package does
not split the current stop generation or introduce per-root stale-handle
epochs.
