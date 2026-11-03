# Implementation reference

These documents describe verified compiler/SDK behavior and representation details.
The [handbook](../pascal/README.md) teaches the language; the
[grammar](../specs/syntax.md), [CLR mapping](../specs/clr-mapping.md), and
[MSBuild contract](../build/msbuild.md) define its implementation boundaries.
Known baseline omissions and deferred proposals remain in
[language gaps](../future/language-gaps.md).

| Area | Reference |
| --- | --- |
| Compiler API/CLI, output, and basic execution | [Compiler overview](first-program.md) |
| Expressions and structured control flow | [Expressions](expressions-and-control-flow.md) and [typed loops](loops.md) |
| Multiple sources, ownership, and visibility | [Source units](multiple-sources.md) |
| CLR objects, nullable flow, exceptions, disposal | [Objects/resources](clr-objects-and-resources.md) |
| Arrays, indexers, generics, enumeration | [Collections](arrays-and-collections.md) |
| Delegates and captured state | [Functions/closures](functions-and-closures.md) and [nested routines](nested-routines.md) |
| Records, defaults, copies, and equality | [Records](records.md) |
| Unions, payload patterns, and coverage | [Unions/matching](unions-and-matching.md) |
| Generic routines and data | [Routines](generic-routines.md), [types](generic-types.md), [inference](type-argument-inference.md) |
| Contracts and managed parameter modes | [Contracts/parameters](contracts-and-parameters.md) |
| Overloads, interfaces, PDBs, and async | [CLR execution features](overloads-interfaces-debugging-and-async.md) |
| Comments and XML sidecars | [XML documentation](xml-documentation.md) |
| Primitive/scalar domains and aliases | [Scalars](scalar-types.md) |
| Shared unit state and initialization | [Globals](global-values.md) |
| Imported events and source metadata | [Events](events.md), [typeof](typeof.md), [attributes](attributes.md) |
| Project SDK and mixed C#/FPAS builds | [MSBuild integration](msbuild.md) |
| CLR comparisons and specification verification | [Acceptance coverage](comparisons-and-acceptance.md) |

## See also

- [Development workflow](../build/development.md)
- [Repository architecture](../architecture/project-layout.md)
- [Tests](../../tests/README.md)
