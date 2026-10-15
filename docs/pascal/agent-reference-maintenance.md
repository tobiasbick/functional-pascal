# Maintaining the compact FPAS reference

This is the permanent maintenance guide for [agent-reference.md](agent-reference.md).
Keep it alongside the reference, independently of temporary wish lists or plans.

## Purpose

FPAS cannot be assumed to be part of an agent's pretrained knowledge. The reference
provides the language context needed to write correct FPAS: implemented rules,
Pascal/Delphi/Ada pitfalls, and complete tested examples. It is the starting point
for authoring; the [handbook](README.md) supplies detailed semantics and API catalogs.

## Size and usefulness

Keep the reference as small as possible while supplying enough context for reliable
authoring. Put essential rules and examples in the reference; link to detailed APIs
and explanations. Remove repetition and stale material when adding coverage.

About 10,000 tokens is a useful orientation, not an acceptance limit. A larger file
is appropriate when necessary to explain the implemented language. Preserve useful
rules, clarity and examples rather than cutting them to meet a token count.
Measure substantial edits with the target model's tokenizer when available;
otherwise label the tool's token estimate as an estimate. Report useful measurements
with the change rather than keeping a cached count in the reference.

## Update and verify

1. Check the reference when implemented syntax, type checking/inference, generics,
   evaluation, units/imports, or an authoring workflow/API used in its examples
   changes. Update affected prose, comparison rows and examples together;
   otherwise confirm that the reference remains accurate.
2. Describe implemented behavior. Approved planning decisions enter the reference
   when implementation and verification are complete. Consult the owning handbook
   pages, [grammar](../specs/grammar.ebnf) and current compiler/runtime when they
   disagree; follow the language-change boundary in [AGENTS.md](../../AGENTS.md).
3. Keep every fence tagged `pascal` and containing a uniquely named, complete program
   or unit. Programs use explicit imports and meaningful assertions; units retain
   their described companion/project setup. Keep stable fence ids and the
   [documentation checker instructions](tools/documentation-examples.md), including
   `run` for the assertion programs. Rejection examples belong in prose.
4. Reuse the [CLI reference test](../../crates/fpas-cli/src/main_tests/examples/agent_reference.rs):
   run `cargo test -p fpas-cli agent_reference_examples_check_and_run`, check example
   formatting, and validate changed links and `git diff --check`. The test checks
   every source and runs each program through the documented project model.
   The normal FPAS suite additionally checks all handbook examples through
   `tools/doc-examples/doc-examples.fpasprj`.
5. Keep this guide discoverable from the reference and the repository's Definition
   of done. Keep maintenance instructions here so the authoring reference spends
   its context on the language. Record validation results in the delivery report.
