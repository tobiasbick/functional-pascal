# Documentation examples

The handbook and agent reference use ordinary Markdown with Pascal examples.
Complete source fences are compiled directly by LanguageSpecificationTests;
programs execute with assertions on output and behavior, and unit examples
compile as libraries.

## Complete sources and companions

A complete fence begins with a program, async program, or unit header and includes
its imports, declarations, and current named terminator. Keep programs independent.
Use a stable descriptive program name; each new program needs an execution
assertion in the documentation runner.

Companion fences share a label such as `pascal project=geometry`. A project has
one program and its units. The runner supplies the labeled units to that program
and compiles library examples with their unit companions. Files are not silently
rewritten or concatenated into a different source language.

## Syntax fragments

Use `pascal fragment` when illustrating a declaration, expression, or statement
inside an existing scope. A fragment is explicitly incomplete: its surrounding
bindings/imports are described by the article or a linked complete example.
The complete-source runner does not invent those bindings or claim that fragment
execution was checked. Formal grammar uses text/ebnf fences rather than Pascal.

## Controlled execution

Examples assert deterministic results. File examples run in isolated directories;
ReadWorld checks an existing file and missing-file cleanup. The HTTP example
uses a local rejecting proxy to verify its exception path without external
network access. Timer examples do not assert exact tick counts.

Link, navigation, fragment-marking, and copied-input checks prevent broken or
stale documentation from silently passing. AgentReferenceTests also enforces
the compact reference's exact o200k_base budgets. Follow the
[development workflow](../../build/development.md) to rebuild copied inputs
and run the focused documentation tests.

## See also

- [Complete examples](../examples/README.md)
- [Agent reference maintenance](../agents/maintenance.md)
- [Source comments and XML documentation](../language/basics/comments.md)
- [Acceptance coverage](../../implementation/comparisons-and-acceptance.md)
