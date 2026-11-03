# Compiled programs

FPAS compiles directly to CIL, CLR metadata, and optional debug/documentation
sidecars. A managed executable DLL runs through the .NET host; a library exposes
ordinary CLR types/methods to FPAS and other .NET languages.

## Output files

| File | Purpose |
| --- | --- |
| .dll | Implementation assembly with actual CLR signatures and bodies |
| .pdb | Assembly-matching portable source/debug metadata, enabled by default |
| .runtimeconfig.json | Executable host/runtime selection |
| .deps.json and copied dependencies | SDK-managed runtime asset resolution |
| .xml | Optional API documentation associated with the assembly |

Direct CLI compilation writes its documented assembly/PDB/runtime configuration
and optional XML outputs. SDK projects additionally manage dependency/host files
and transitive runtime copying. Libraries have no executable entry point or
runtime configuration. Disabling a previously generated output removes stale
compiler/project-owned sidecars according to the output contract.

## Runtime behavior

The CLR owns execution, garbage collection, exceptions, and task infrastructure.
Leaving a function does not synchronously reclaim every referenced object.
Use [resource declarations](../language/error-handling/README.md) for scheduled
IDisposable/IAsyncDisposable cleanup of external resources.

Language records/unions/domains retain their documented nominal CLR identities.
Fpas.Core supplies checked metadata contracts needed for language types and
aliases; it is not an implicit source namespace. The SDK supplies and deploys its
matching support assembly. Standalone deployment must include required external
assemblies, including Core when its metadata is inspected. Source imports do not
replace runtime dependency resolution.
The [standalone SDK walkthrough](../../build/standalone-sdk.md#copy-runnable-output)
demonstrates copying complete FPAS and C# application outputs and running them
without the consumer sources, package feed, or compiler tools.

Keep source files and matching symbols for source debugging. Generated closure/
async methods carry the corresponding portable metadata. CLR execution does not
imply sandboxing, process isolation, or a separate language scheduler.

## See also

- [CLR mapping](../../specs/clr-mapping.md)
- [Projects](projects.md)
- [Compiler CLI](cli.md)
- [Debugging](../tools/debugging.md)
