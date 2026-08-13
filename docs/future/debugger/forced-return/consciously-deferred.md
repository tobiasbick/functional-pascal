# Consciously deferred

The following capabilities remain outside the bounded forced-return slice.

- Returning the program entry frame or a spawned task's entry frame, including
  forced task completion, retained task results, waiter wake-up, or failure
  replacement.
- Selecting any non-active stack frame, discarding multiple younger frames, or
  preserving selected younger-frame values during unwind.
- Replacing a return value after the callee has completed or retaining a
  completed frame solely for later editing.
- Applying forced return to waiting, sleeping, suspended, cancelled, completed,
  or non-selected peer tasks.
- Recovering from a runtime error, suppressing a diagnostic, or converting a
  failed task into a successful return.
- Running source instructions, cleanup code, user callbacks, scheduler turns,
  or hosted I/O as part of debugger-controlled frame exit.
- Inferring a result type from a display name, current register value, return
  opcode, function name, or call destination when portable metadata is absent.
- Returning Dynamic, first-class function, task, capture-cell, opaque hosted,
  or other identity-bearing values in the initial package.
- Changing the instruction pointer without returning, restarting a frame,
  retrying a call, jumping to a source line, or preserving locals across a
  restart.
- Data breakpoints or breakpoint actions that force a return or otherwise
  modify control flow.
- Filling descendants of uninitialized aggregate storage, synthesizing missing
  capture cells, or treating absent parameters as user-initializable.
- Any FPAS language syntax, semantic, or normative specification change.

Each later package must define stable IDs, frame/task ownership, portable type
proof, cleanup and scheduler behavior, atomicity, protocol parity, resource and
privacy bounds, negative tests, and its own resume log before implementation.
