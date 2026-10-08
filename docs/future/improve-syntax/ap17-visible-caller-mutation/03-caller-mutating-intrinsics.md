# AP17.3: Caller-mutating intrinsics

Package: [AP17: Visible caller mutation](README.md)

Status: complete.

## Result

Native `Items.Push(Value)` and `Items.Pop()` use the shared writable-storage
checks. Their implicit receiver has no `var` marker or extra parentheses.
Writable direct, field, array-element, imported-variable and forwarded-reference
receivers are valid. Constants, read-only values, properties, dictionary entries,
string characters and computed receivers are rejected.

The root and indices are fixed once before explicit arguments. Mutation uses
that storage after argument evaluation. `Push` appends its read-only Value and
returns no value; `Pop` removes and returns the final element. Copy-on-write
preserves other values sharing the array. Completed writes survive failures and
`try` exits; writable receivers cannot cross `go` boundaries.

The native catalog provides the only public form, including
`Items.Push(Value := 3)`, without helper imports. Own free routines with a first
`var` parameter remain ordinary explicitly marked calls. Checking and lowering
share reference storage paths with ordinary `var` parameters.

## Regression coverage

Compiler, sema and FPAS tests cover every writable/rejected storage form,
forwarding, shared arrays, index/evaluation order, mutation during arguments,
failures, `try`, named Push, task rejection and returned Pop chains.
See [array mutation](../../../pascal/language/types/array/mutating.md).
