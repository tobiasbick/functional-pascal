# FPAS projects and publishing

Depends on the [managed binding](managed-binding.md). Integrate references,
native assets and final linking through the existing
[project SDK](../../../sdk/Fpas.NET.Sdk/README.md) and
[publishing workflow](../../build/publishing.md).

## Project integration

- [ ] Add the binding to the repository solution and library/package build.
  GUI projects opt in through normal CLR project/package references. Keep CUI
  out of compiler/CLI startup and out of terminal-only applications.
- [ ] Add an internal FPAS integration fixture that opens a window and exercises
  a button, checkbox and text field through the binding. This proves emitted
  FPAS CIL and native interop before MVU extraction; it is not a second public
  application programming model.
- [ ] Resolve native assets from the selected RID, including when the managed
  binding is reached through another project/package. Avoid absolute build-machine
  paths, dependence on the working directory and manual consumer copy steps.
- [ ] Make ordinary build/run use the matching shared library automatically.
  Report unsupported RIDs or missing assets while building.

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

- [ ] Build and run the FPAS fixture through the packaged/local-feed SDK, not
  just through references to source-tree build output.
- [ ] Publish and launch from another working directory with no source-tree
  library search overrides. Check native asset selection and license delivery.
- [ ] Copy only the NativeAOT executable to an otherwise empty directory. Without
  a .NET installation or adjacent CUI library, it must render, accept the test
  interactions and leave a native extraction directory empty.
- [ ] Establish Linux evidence first and Windows evidence separately. Use native
  target runners for platform-dependent AOT/link/window checks.

Update the existing build/publishing references with the implemented procedure.
The final public FPAS example uses the [MVU GUI host](gui-host.md).
