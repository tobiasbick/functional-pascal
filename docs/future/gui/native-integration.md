# Native CUI integration

Provide reproducible shared and static native assets from one pinned single
header. This stage supplies `Fpas.Gui.Cui`; it does not add language features.

## Native build

The pinned header, provenance, checksum, update procedure and source notices are
owned by [third-party/cui/UPSTREAM.md](../../../third-party/cui/UPSTREAM.md).

The [native build](../../../src/Fpas.Gui.Cui/native/README.md) compiles the header
in one translation unit and supports separate shared/static build directories.
Linux x64 builds, public symbols and headless consumer link/render tests are
verified for both variants. The static consumer has no shared CUI dependency.
Windows build definitions are present; Windows execution remains unverified.
The [managed binding](../../../src/Fpas.Gui.Cui/README.md) builds the native
module through MSBuild; Linux window/input/capture tests cover software and OpenGL.

## Remaining integration

- [ ] Add remaining native control/callback/wakeup helpers required by the managed binding
  under `src/Fpas.Gui.Cui/native/`. Keep C# binding code under `src/Fpas.Gui.Cui/`
  and continue using the single vendored header in `third-party/cui/`.
- [ ] Include the retained CUI, stb, Unicode and embedded font/icon notices in
  native asset and binary distributions. Builds use the committed header without
  downloading or regenerating it.
- [ ] Integrate native asset production into the complete repository/SDK
  distribution build; the managed project's local MSBuild invocation is available.
- [ ] Include native assets in the matching FPAS SDK/library distribution.
  Ordinary consumers receive prebuilt assets; building FPAS from source requires
  the documented native build tools. NativeAOT publishing needs its normal
  target-platform linker and development libraries.

## Acceptance

- [ ] Windows: independently build both variants and run the same basic window
  check. Verify calling convention, exported names and static-library inputs.
- [ ] Native distribution includes both linkage forms and all notices. Missing
  RID assets produce a precise build diagnostic rather than a runtime download.

Continue with the [managed binding](managed-binding.md). Packaging into consuming
FPAS projects belongs to [FPAS publishing](fpas-publishing.md).
