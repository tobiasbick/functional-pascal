# FPAS projects and publishing

Depends on the [managed binding](managed-binding.md). Integrate references,
native assets and final linking through the existing
[project SDK](../../../sdk/Fpas.NET.Sdk/README.md) and
[publishing workflow](../../build/publishing.md).

The [binding reference](../../../src/Fpas.Gui.Cui/README.md#fpas-integration-fixture)
owns the implemented source-project integration and internal FPAS fixture.
The binding and GUI tests belong to the repository solution. Linux checks use a
freshly packaged FPAS SDK and a normal binding project reference; native binding
packages and the publishing modes below remain open.

## Remaining project integration

- [ ] Include the binding, native assets, notices and transitive build/link
  targets in the library/package distribution. Keep GUI opt-in through normal
  CLR references and keep CUI out of compiler/CLI startup and terminal-only apps.
- [ ] Resolve native assets from the selected RID, including when the managed
  binding is reached through another project/package. Avoid absolute build-machine
  paths, dependence on the working directory and manual consumer copy steps.
- [ ] Extend automatic shared-library delivery from source-project references
  to packaged consumers. Report unsupported RIDs or missing assets while building.

## Publishing modes

- [ ] Preserve the current `fpas publish` command and mode selection; NativeAOT
  remains its default. Use standard MSBuild/.NET publishing properties rather
  than introducing a parallel GUI deployment command.
- [ ] Framework-dependent, apphost and self-contained modes deliver the matching
  shared CUI library beside the managed application.
- [ ] Managed single-file mode embeds/extracts the native library through .NET's
  supported publishing mechanism. Include required native data and notices.
- [ ] NativeAOT uses direct P/Invoke and the matching static CUI archive at the
  consuming executable's final link. Deliver the targets transitively so FPAS
  consumers do not need handwritten imports or linker commands.
- [ ] Supply the target OS libraries needed for final linking. Avoid a shared
  CUI dependency or a bundled-library extraction fallback in the AOT result.
- [ ] Keep GUI assets/notices inside the existing owned publish-output and
  signing/manifest workflow. License notices and optional debug symbols can
  accompany the executable; they are not runtime dependencies.

## Acceptance

- [ ] Build and run the FPAS fixture through packaged native binding assets as
  well as the packaged/local-feed SDK.
- [ ] Publish and launch from another working directory with no source-tree
  library search overrides. Check native asset selection and license delivery.
- [ ] Copy only the NativeAOT executable to an otherwise empty directory. Without
  a .NET installation or adjacent CUI library, it must render, accept the test
  interactions and leave a native extraction directory empty.
- [ ] Establish Linux evidence first and Windows evidence separately. Use native
  target runners for platform-dependent AOT/link/window checks.

Update the existing build/publishing references with the implemented procedure.
The final public FPAS example uses the [MVU GUI host](gui-host.md).
