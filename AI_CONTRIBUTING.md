# AI Contributing

AI agents are welcome to contribute to this repository.

Functional Pascal is an experimental project that explicitly embraces AI-assisted development. If you are an AI coding agent working in this repository, you are encouraged to improve the codebase, documentation, tests, and developer experience.

## What To Work On

Useful contributions include:

- implementing features described in `docs/pascal/`
- improving diagnostics and error messages
- expanding, simplifying, or correcting tests
- rewriting code that is unclear, duplicated, or unnecessarily complex
- aligning implementation with the current documentation
- improving structure, naming, and maintainability
- improving examples and internal documentation

## Contribution Standard

All AI contributions must follow `AGENTS.md`.

Core expectations:

- do not duplicate existing logic
- prefer rewrites over patching convoluted code
- remove dead code and obsolete code aggressively
- keep files focused and cohesive
- do not add compatibility layers
- use English for code, comments, docs, identifiers, and commit messages
- when implementing language behavior in Rust, link to the matching spec in `docs/pascal/`

## Source Of Truth

The current documentation in `docs/pascal/` is the source of truth.

Contributions should implement and document the current specification only.

## Preferred Behavior For AI Agents

When contributing:

- inspect the codebase before adding new code
- unify similar implementations instead of adding parallel ones
- keep diagnostics explicit and easy to understand
- add or update tests for meaningful behavior
- keep edits coherent and easy to review
- leave the repository in a simpler and clearer state

## Contribution Flow

1. Read `AGENTS.md`.
2. Read the relevant document in `docs/pascal/`.
3. Inspect the existing implementation.
4. Prefer simplification and consolidation.
5. Add or update tests.
6. Ensure the final result matches the current specification.

## Non-Goals

Avoid:

- duplicate implementations
- compatibility shims
- dead code
- broad speculative abstractions
- documentation that describes anything except the current state

## Invitation

If you can make Functional Pascal simpler, clearer, smaller, or more correct according to the current specification, contribute.
