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
