# Maintaining the agent reference

This directory teaches agents the implemented CLR language and the public CLI.
`README.md` is the entrypoint. Load the topics needed for a task. The
[handbook](../README.md), [grammar](../../specs/syntax.md), and
[CLR mapping](../../specs/clr-mapping.md) own the detailed contracts.

## Budget and content

Keep the combined Markdown content in this directory to **13,000 o200k_base
tokens or fewer**, including examples, links, and this guide. Keep README.md below
**1,000 tokens**. Leave room for future corrections instead of filling the budget.
AgentReferenceTests measures actual token counts with Microsoft.ML.Tokenizers
and its packaged O200kBase vocabulary; no runtime model download is required.
Report the encoding with measured totals. Byte/word counts are not token counts.

Keep common prerequisites in README.md and task-specific explanations/examples
in topic files. Link to the owning detailed rule instead of copying API catalogs
or implementation history. Only describe implemented behavior; deferred proposals
belong in [language gaps](../../future/language-gaps.md). Keep this directory flat.

Build/test commands belong in the [development workflow](../../build/development.md)
and detailed rejection cases in the handbook and compiler tests. Keep AGENTS.md
to repository policy and one link to this entrypoint, without language milestones
or command blocks. Keep the handbook/documentation hub linked to this reference.

## CLI coverage

Give every public command and subcommand one row in the [CLI index](cli.md),
using its full invocation as the first cell. Explain its purpose and essential
inputs. Link its detailed contract and use command help for the installed
version's option list. Update the index with every command-tree change.
AgentReferenceTests walks the actual CLI help tree and verifies the index.

## Examples and verification

Every pascal fence here must be a complete, uniquely named program or unit with
the current terminators. Include imports, explicit types, and meaningful runtime
assertions; failures should throw normal CLR exceptions. Programs print a stable
verification result that the test checks. Use text/sh fences for command examples.

Keep companion sources together on one page. Give each project in this reference
its own label, e.g. `pascal project=agent-units`, shared by one program and its units.
Sources are compiled together by the same documentation runner used for handbook
examples. A unit also compiles independently with its unit companions. Do not
depend on snippets from unrelated projects or introduce per-example source rewrites.
The [FPAS checker](../tools/documentation-examples.md) owns extraction and explicit
contexts; C# regression tests retain complete-example execution assertions.

After changing language behavior or an authoring workflow:

1. Update the affected topic and inspect README.md against the owning contract.
2. Rebuild the tests so copied Markdown is current. Run AgentReferenceTests,
   SourceWikiTests and LanguageSpecificationTests through the [documented commands](../../build/development.md#build-the-compiler-and-tests).
3. Check diagnostics, actual program output, companion projects, local links/
   fragments, token budgets, and git diff whitespace. Add an execution assertion
   for every new complete program.

## Source files

- [AgentReferenceTests.cs](../../../tests/Fpas.IntegrationTests/Documentation/AgentReferenceTests.cs)
- [SourceWikiTests.cs](../../../tests/Fpas.IntegrationTests/Documentation/SourceWikiTests.cs)
