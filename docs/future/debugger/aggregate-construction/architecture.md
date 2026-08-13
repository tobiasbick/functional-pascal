# Architecture

## Current foundation

The existing debugger already provides the required safety boundaries:

- `DebugSession::set_expression_with_limits` resolves a textual target,
  evaluates detached expressions, validates metadata, and performs one commit.
- `mutation/transition/` recognizes exact qualified single-payload transitions.
- Debug expression calls can construct enum, `Result`, and `Option` values.
- JSONL owns the external protocol, DAP maps editor requests to JSONL, and the
  VS Code extension sends DAP requests.

The new package generalizes metadata-driven construction without changing the
existing assignment operations.

## Operation flow

### Discovery

1. Parse the textual target and optional frame ID.
2. Resolve its declared debug type without mutating the live program.
3. Require enum, `Result`, or `Option` metadata.
4. Return canonical variants in metadata order, including canonical field names
   and field type display names.

### Construction

1. Parse the target and all field-expression strings.
2. Resolve the current mutable target and its declared wrapper type.
3. Resolve the exact requested variant from portable metadata.
4. Validate that request fields exactly match declared fields.
5. Evaluate field expressions once in declaration order against the unchanged
   stop snapshot under one shared limit budget.
6. Validate each detached field value against its declared debug type.
7. Build one detached complete enum, `Result`, or `Option` value.
8. Revalidate the target generation and commit one live root.
9. Return the ordinary debugger value summary and canonical selected variant.

No intermediate payload becomes observable.

## Protocol contracts

### JSONL discovery

Command: `variant.describe`

```json
{
  "request_id": 20,
  "command": "variant.describe",
  "arguments": {"frame_id": 3, "target": "Selected"}
}
```

Successful body:

```json
{
  "target": "Selected",
  "type_name": "Choice",
  "variants": [
    {"name": "Choice.Empty", "fields": []},
    {
      "name": "Choice.Pair",
      "fields": [
        {"name": "Left", "type_name": "Integer"},
        {"name": "Right", "type_name": "Integer"}
      ]
    }
  ]
}
```

### JSONL construction

Command: `variant.construct`

```json
{
  "request_id": 21,
  "command": "variant.construct",
  "arguments": {
    "frame_id": 3,
    "target": "Selected",
    "variant": "Choice.Pair",
    "fields": {"Left": "NextLeft()", "Right": "2"}
  }
}
```

`fields` maps canonical or ASCII-case-insensitive field names to debugger
expression strings. The response uses the established mutation result keys and
adds `variant` with canonical spelling.

### DAP

- `fpas/variantDescribe` maps `frameId` and `target` to `variant.describe`.
- `fpas/variantConstruct` maps `frameId`, `target`, `variant`, and `fields` to
  `variant.construct`.
- A successful construct response is followed by a negotiated DAP
  `invalidated` event for the `variables` area.
- Discovery does not emit invalidation.

### VS Code

Command: `functionalPascal.debug.constructVariant`

The command selects the active frame, prompts for a textual target, calls
`fpas/variantDescribe`, presents a variant Quick Pick, prompts once for every
declared field expression, and calls `fpas/variantConstruct`. Programmatic
command arguments may provide `frameId`, `target`, `variant`, and `fields` to
support extension-host tests and automation.

## Intended file layout

```text
crates/fpas-vm/src/vm/debug/
  session/
    variant.rs                   — NEW: public session discovery/construction operations
  mutation/
    variant/
      mod.rs                     — NEW: focused variant-construction module root
      model.rs                   — NEW: descriptors and construction request/result models
      metadata.rs                — NEW: enum/Result/Option metadata normalization
      construct.rs               — NEW: exact-field validation and detached value construction
      diagnostics.rs             — NEW: stable construction diagnostics and hints
    transition/
      suffix.rs                  — MODIFY: reuse normalized variant metadata
      mod.rs                     — MODIFY: reuse shared complete-value construction
    mod.rs                       — MODIFY: module declarations and focused re-exports
  session.rs                     — MODIFY: declare the focused session module only
  types.rs                       — MODIFY: add only required stable error kinds

crates/fpas-debug/src/
  jsonl/
    server/
      variant.rs                 — NEW: JSONL argument parsing and response encoding
    server.rs                    — MODIFY: dispatch two JSONL commands
  dap/
    server/
      variant.rs                 — NEW: DAP custom-request translation
      mutation.rs                — MODIFY: recognize construct result and invalidation
    server.rs                    — MODIFY: dispatch two DAP custom requests

editors/vscode/
  src/debugger/
    variantConstructionCommand.ts — NEW: discovery, Quick Pick, field prompts, request
    adapter.ts                     — MODIFY: register the command
  test/debugger_host/
    variant_construction.ts        — NEW: real extension-host coverage
  test/debugger_host.ts            — MODIFY: invoke the new coverage
  package.json                     — MODIFY: contribute the command

tests/debugger/fixtures/
  variant_construction.fpas      — NEW: shared fieldless/multi-field fixture

crates/fpas-vm/src/vm/debug/tests/
  variant_construction/          — NEW: metadata, construction, atomicity, limit tests

crates/fpas-debug/tests/
  variant_construction.rs        — NEW: JSONL contract tests
  dap_variant_construction.rs    — NEW: DAP mapping and invalidation tests

docs/pascal/tools/debugger.md    — MODIFY after implementation: current behavior only
docs/future/debugger/            — MODIFY after completion: remove DBG-D01 and archive plan status
```

No existing large file receives construction logic. The shared metadata module
prevents the existing qualified-transition implementation and the new operation
from developing separate variant rules.
