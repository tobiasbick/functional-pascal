# Language gaps and deferred extensions

The [baseline comparison](../baseline-comparison.md) leaves no confirmed baseline
compiler or MSBuild omission open for net10.0/AnyCPU on Windows and Ubuntu/WSL.
Record newly demonstrated omissions here with their owning rule and reproducer.
Practical maintenance limits belong in [dogfooding gaps](dogfooding-gaps.md);
guides remain tracked in [documentation gaps](documentation-gaps.md).

Source interfaces are implemented. Their syntax, inheritance, record
implementation, and CLR representation are specified in
[interfaces](../pascal/interop/interfaces.md), the [grammar](../specs/syntax.md),
and [CLR mapping](../specs/clr-mapping.md). They are no longer a deferred proposal.

## Open decision: lifted nullable numeric operators

Numeric lifted nullable arithmetic is unsupported; see
[nullable types](../pascal/language/types/nullable-types.md). The baseline defines
nullable storage, flow, validation, and coalescing, but does not give a complete
lifted-operator contract. Operator selection, null results, comparisons, and
checked arithmetic need an agreed contract before implementation. This decision
is separate from the completed interface work.

## Scope boundary

Source event declarations, expression-tree conversion, user-authored class
hierarchies, custom awaiters, async iterators, unsafe/byref-like features,
operator declarations, automatic extension-method discovery, and interface
variance/default bodies remain outside the agreed scope. Publishing/AOT/signing,
native apphosts, additional platform/framework coverage, LSP, queries, VS Code,
and Visual Studio project-system integration are separate toolchain proposals.
An unsupported item mentioned in an old milestone does not create a new task.
