---
name: fpas-projects
description: Create or edit FPAS .fpasprj and .fpasworkspace manifests, dependencies, exports, test bundles, and project discovery.
---

# FPAS projects

Keep sources and manifests authoritative. Shared repository rules and completion
checks live in [AGENTS.md](../../../AGENTS.md).

## Read for the task

| Task | Reference |
|------|-----------|
| Project kind, main source, dependencies, exports | [Projects](../../../docs/pascal/program-structure/projects.md) |
| Workspace members and discovery | [Workspaces](../../../docs/pascal/program-structure/workspaces.md) |
| CLI discovery, build and run inputs, flags | [CLI](../../../docs/pascal/program-structure/cli.md) |
| Test discovery and sidecars | [Test runner](../../../docs/pascal/std/testing/test.md) |

Use focused command help for the operation at hand, such as `fpas init --help`
or `fpas test --help`.

## Manifest workflow

1. Inspect neighboring manifests and the owning source tree. For a new scaffold,
   use the matching initializer:

   ```text
   fpas init project <name>
   fpas init library <name> --unit <unit-name>
   fpas init workspace <name>
   ```

   Wire existing projects by editing their manifests. Working layouts:
   [monorepo](../../../examples/pascal/monorepo/monorepo.fpasworkspace) and
   [regression suite](../../../tests/suite.fpasprj).
2. Set the project kind and source selection for its purpose. Program projects
   name their entry source; libraries contain units; test bundles select test
   programs and any helper units.
3. Declare dependencies on each consumer. Project paths resolve relative to the
   consumer's manifest; workspace dependencies name a member's `project.name`.
   The workspace lists members. Consumers import exported units with `uses`;
   library `[exports].units` controls their public unit surface.
4. Use [fpas-authoring](../fpas-authoring/SKILL.md) when editing `.fpas` sources.

## Verify manifests

Pass explicit paths when discovery is ambiguous:

```text
fpas check <project-or-workspace>
fpas test <test-bundle> --list
fpas test <test-bundle>
```

Inspect the test list to confirm that added paths are selected. Run interactive
programs only when their execution is part of the task; compilation can be
checked with `fpas check`.
