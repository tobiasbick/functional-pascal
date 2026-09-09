# Human overload

> Written under gentle but unmistakable human pressure after someone had to
> stop and ask whether a particular block wanted `end`, `end;`, or `end.`.
> The compiler declined to comment, so this document was compelled into
> existence instead.

## Consistent `end` punctuation

FPAS currently uses `end`, `end;`, and `end.` depending on what is being
closed and what follows it. Although each form is individually familiar from
Pascal, choosing between them adds a small amount of context-sensitive mental
overhead whenever a block is finished.

The current preference is to require punctuation after every `end`: use
`end;` for nested constructs and declarations, and reserve `end.` for the end
of a compilation unit. Under that model, a bare `end` would no longer be valid.

The decision remains open. Before changing the language, we should consider
readability, nested control-flow syntax, formatter behavior, migration cost,
and whether the extra punctuation actually reduces human effort in real FPAS
programs—or merely gives the semicolon more opportunities to feel important.
