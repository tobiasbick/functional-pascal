# Workspaces

A workspace groups multiple projects, similar to a Visual Studio solution. The `acme-suite` / `apps/portal` member paths in the sample below reuse the same **illustrative** monorepo from [Projects — library dependency example](projects.md#example-program-with-a-library-dependency); see [`examples/pascal/monorepo/`](../../../examples/pascal/monorepo/) for a checked-in workspace.

Define a `.fpasworkspace` file in TOML format:

```toml
[workspace]
name = "acme-suite"
members = [
  "apps/portal/portal.fpasprj",
  "libs/acme-utils/acme-utils.fpasprj"
]
```

| Field | Required | Description |
|---|---|---|
| `name` | Yes | Workspace name. Any non-empty string. |
| `members` | Yes | Array of paths to `.fpasprj` files, relative to the workspace file or absolute. |

`fpas check` with no path loads the sole `.fpasworkspace` in the current directory and checks every member project. `fpas run` with no path runs the sole program member when a workspace is present; otherwise pass a `.fpasprj` explicitly or rely on a single project file in the current directory.

Cross-project dependencies use `[dependencies].projects` or `[dependencies].workspace` on each
consumer `.fpasprj`; the workspace file does not make every member an implicit dependency.
Workspace checks share the normal compiled-unit pipeline, so an unchanged library unit's
source-adjacent `.fpascu` can be reused by multiple members.

## See also

- [Projects](projects.md)
- [CLI](cli.md)
