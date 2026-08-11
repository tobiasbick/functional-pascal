# Workspaces

A workspace groups multiple projects, similar to a Visual Studio solution. The `acme-suite` / `apps/portal` member paths in the sample below reuse the same **illustrative** monorepo from [Projects — library dependency example](projects.md#example-program-with-a-library-dependency); see [`examples/pascal/monorepo/`](../../../examples/pascal/monorepo/) for a checked-in workspace.

`fpas init workspace <name>` creates a complete workspace with one program and
one library member. See [Initializing projects and workspaces](initializing.md).

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

`fpas check` with no path loads the sole `.fpasworkspace` in the current
directory and checks every member project. `fpas run <workspace.fpasworkspace>`
and `fpas run` with a discovered workspace run its sole program member. Both
forms error when the workspace has zero or multiple program members. Without a
workspace, a pathless run relies on a single project file in the current
directory.

Cross-project dependencies use `[dependencies].projects` or `[dependencies].workspace` on each
consumer `.fpasprj`; the workspace file does not make every member an implicit dependency.
Workspace checks share the normal compiled-unit pipeline, so an unchanged library unit's
source-adjacent `.fpascu` can be reused by multiple members.

`fpas build <workspace.fpasworkspace>` processes every member in manifest
order. Program members produce or reuse `<project.name>.fpascp` beside their
own `.fpasprj`; library members build source-adjacent `.fpascu` files; test
members build helper-unit sidecars and validate their test programs.

Running a workspace program produces or reuses that member's
`<project.name>.fpascp` beside its `.fpasprj`, then executes the validated
image.

`fpas build --executable <workspace.fpasworkspace>` requires exactly one
program member and writes the native application beside the workspace
manifest. Its default base name is `workspace.name`; `--name <name>` overrides
that output name. Zero or multiple program members are errors.

## See also

- [Initializing projects and workspaces](initializing.md)
- [Projects](projects.md)
- [CLI](cli.md)
