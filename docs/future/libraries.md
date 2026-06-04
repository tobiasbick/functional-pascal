# Future: Libraries

The project kind `library` and `[dependencies].projects` support source-level reuse today:

- Define units in a `kind = "library"` project.
- Consume them from a `program` project via `dependencies.projects` and `uses`.

Still planned for later versions:

- Public API visibility and explicit export rules beyond `private`.
- Precompiled library artifacts.
