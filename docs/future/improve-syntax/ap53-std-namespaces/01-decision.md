# AP53.1: Record the namespace contract

Package: [AP53](README.md). Status: complete, documentation only.

## Deliverable

Decisions Q68-Q71 in the package README record the accepted organization,
migration scope, document ownership and planning-only authorization.
The binding [specification](../../../specs/std-namespaces.md) owns the public
mapping and future allocations. It does not describe those moves as delivered.

## Evidence and boundaries

The intrinsic catalogue and source export manifest establish the current
public units. Existing nested imports show that the migration needs consistent
catalogues and consumers, not new namespace syntax. `Std.Conv` currently
mixes four text-formatting routines with `IntToReal`; these have separate
destinations with unchanged behavior.

The future directory already contains a Std roadmap and feature plans. Keep
those owners and link to the specification instead of creating a second
future-library plan. Bind contributors through `AGENTS.md` and the standard
library's contributor documentation. Current API examples retain working
imports until AP53.2 implements the migration.

## Affected documents

| Path | Purpose |
| --- | --- |
| `AGENTS.md` | Require the specification when adding, moving, splitting or renaming standard units |
| `docs/specs/std-namespaces.md` | Define canonical homes, domain boundaries and future extension rules |
| `docs/pascal/std/README.md` | Link contributor policy without changing available APIs |
| `docs/pascal/program-structure/units.md`, comments in `docs/specs/grammar.ebnf` | Correct the outdated two-segment-only description using existing nested imports; grammar productions stay unchanged |
| `docs/future/std-roadmap.md` | Keep capability priorities and make AP53 precede buffers |
| `docs/future/fpas2-language-directions.md` | Record the selected namespace work and buffer prerequisite |
| `docs/future/README.md` | Index the enduring policy and active migration |
| `docs/future/improve-syntax/README.md` | Register AP53 and reserve its decision IDs |
| `docs/future/improve-syntax/ap53-std-namespaces/` | Record decisions, scope, dependencies and acceptance |
| `docs/future/networked-applications/` | Align future database, protocol and operational allocations |
| `docs/future/webdav.md`, `ftp.md`, `hardware-information.md` | Assign future homes while preserving deferred/idea status |
| `docs/future/graphics.md`, `graphics-alternative.md`, `shared-ui.md` | Align graphics, audio, window, input and UI homes without choosing their implementations |

## Acceptance and verification

Every currently exported standard unit has a mapping, including units whose
name stays the same. The specification separates binding placement from
availability and implementation permission. Roadmap, AP and feature plans
refer to that one policy. The package has a concrete migration scope and
verification criteria, with future implementation work left unchecked.

Review relative links, catalogue coverage, decision/status consistency and
`git diff --check`. This work package requires no builds or tests.
