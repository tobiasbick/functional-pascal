# Maintaining the agent reference

This directory is the language reference for agents unfamiliar with FPAS.
[README.md](README.md) is the entrypoint; topic files are loaded for the task.
The [handbook](../README.md) owns detailed semantics and API catalogs.

## Budget and scope

Keep all Markdown files in this directory together below **10,000 tokens**,
including this guide. Aim for 8,000–9,000 to leave editing room. The entrypoint
must stay below 1,000 tokens; topic files should usually be 800–1,500 tokens.
Measure the actual files, including examples and links, with `o200k_base` or the
target model's tokenizer and name the encoding in the delivery report. Label an
estimate explicitly when no tokenizer is available; do not claim the limit is
verified from byte or word counts.

Keep common prerequisites in the entrypoint and task-specific rules beside their
examples. Links must name when to read their target. Prefer a short language rule
and a focused handbook link over duplicating an API contract. `fpas doc` supplies
exact signatures. Preserve core syntax and pitfalls that an unfamiliar agent
cannot infer from API lookup alone. Keep this directory flat for predictable
discovery by the example test.

## Changes and verification

When implemented language behavior or a demonstrated authoring workflow changes,
update the affected topic and inspect the entrypoint. Resolve discrepancies
against the owning handbook, grammar and current implementation. Unimplemented
proposals stay in `docs/future/`.

Keep Pascal fences as complete, uniquely named programs or units with stable ids.
Runnable examples have meaningful assertions and the `run` instruction; unit
companions share their documented project. Use `text` for command examples.
Detailed rejection examples belong in the relevant handbook page.

The existing `agent_reference_examples_check_and_run` CLI test discovers every
Markdown file here, compiles each Pascal example and runs its programs, including
companions. Run:

```text
cargo test -p fpas-cli --bin fpas agent_reference_examples_check_and_run
fpas run tools/doc-examples/doc-examples.fpasprj
```

Also check example formatting, changed local links, the token budgets and
`git diff --check`. The general documentation checker discovers these files
through its normal handbook selection. Keep the entrypoint linked from
`AGENTS.md`, the authoring skill and handbook, and this guide linked from
`CONTRIBUTING.md`. Shared repository rules stay in those owning documents.
