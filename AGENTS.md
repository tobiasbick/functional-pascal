# AGENTS

## Code Quality Rules

- **No duplication.** Before adding code, check for existing duplicates or similar implementations. Unify and consolidate rather than adding alongside.
- **Rewrite over repair.** This project is work-in-progress. Prefer discarding and rewriting stale or convoluted code over patching legacy. There is no backward compatibility requirement.
- **Keep it lean.** Remove dead code, unused imports, and obsolete modules aggressively.
- **Keep files focused and reasonably small.** Each file should have one cohesive responsibility (one concern/topic). Aim to stay under 500 LOC when practical. Do not split code artificially just to satisfy a line-count target—clarity and cohesion come first. For broad components (for example, a lexer), split by clear sub-responsibilities (such as token definitions, scanning logic, and diagnostics), not by arbitrary size. Use directories to organize related files.
- **No backward compatibility.** We do not want nor need backward compatibility, only accept the current specs. The language is not fixed yet.
- **No legacy or backward references.** When you change something, do not mention old behavior. Document only the current state.

## Code and documentation

- **Links in Rust.** Always add a link to the corresponding documentation under `docs/pascal/` in the Rust source file when it implements part of that spec.

## Error Messages

- **LLM-friendly diagnostics.** Error messages emitted by the compiler, lexer, parser, etc. must be understandable by LLMs. When possible, include a hint showing the correct syntax or idiom.

## Language

- All code, comments, documentation, commit messages, and identifier names **must be in English**.

## Environment

- **Rust**: edition **2024**, `cargo build` / `cargo fmt` / `cargo test --workspace`; sources use `.fpas`.

# 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

# 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

# 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

# 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.
