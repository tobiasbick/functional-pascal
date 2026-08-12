# Consciously deferred

The following work is deliberately outside uninitialized-binding assignment.
It is not implied by the implemented root-assignment capability.

- Writing a field, array index, dictionary entry, enum payload, `Result` value,
  or `Option` value before its complete root exists.
- Skipping, suppressing, or rewriting a later source initializer.
- Creating a missing closure cell or treating an absent parameter or capture as
  a user-initializable source binding.
- Initializing compiler-hidden storage, evaluation-only values, function values,
  task handles, or opaque hosted resources.
- Changing control flow, the instruction pointer, return values, or frame
  lifetime as a side effect of assignment.
- Data breakpoints or breakpoint actions that mutate state.

Any future package that takes one of these items must define new stable IDs,
security and atomicity constraints, protocol coverage, and its own verification
matrix before implementation.
