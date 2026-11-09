# Native CUI integration

Provide reproducible shared and static native assets from one pinned single
header. This stage supplies `Fpas.Gui.Cui`; it does not add language features.

## Source and layout

- [ ] Add the native sources under `src/Fpas.Gui.Cui/native/`: the vendored
  `cui_single.h`, a small implementation/bridge file, CMake configuration and
  complete license notices.
- [ ] Pin [CUI v1.2.0](https://github.com/yappologistic/CUI/releases/tag/v1.2.0),
  commit `7bb08ff7c56eb912fe7133985f8b7fa65098ea2e`. Record header SHA-256
  `c16ae25c41d4cf8164f8ca111c5d7d8ba5fd2e27bb7dd323bd53288f2bff274e`
  and the procedure for deliberately replacing the vendored release.
- [ ] Retain the CUI, stb, Unicode and embedded font/icon license notices in a
  form that also accompanies binary distribution. Regular builds use the
  committed header without downloading or regenerating it.
- [ ] Compile exactly one translation unit with `CUI_IMPLEMENTATION`. Keep
  bridge helpers focused on ABI adaptation, C macros, lifecycle and host wakeup.

## Native assets

- [ ] Use private native module name `fpas_cui`: `libfpas_cui.so` on Linux and
  `fpas_cui.dll` on Windows. Produce the static `.a`/`.lib` for NativeAOT from
  the same source. Keep Windows import libraries separate from static archives.
- [ ] Build shared and static variants into separate directories per RID,
  configuration and linkage. Apply the appropriate export definitions only to
  shared builds; ensure the static archive can link into the final executable.
- [ ] Connect CMake to the repository's MSBuild build and distribution. Let
  CMake handle incremental native recompilation when inputs/options change.
- [ ] Include native assets in the matching FPAS SDK/library distribution.
  Ordinary consumers receive prebuilt assets; building FPAS from source requires
  the documented native build tools. NativeAOT publishing needs its normal
  target-platform linker and development libraries.
- [ ] Use CUI's X11 and Win32 backends with software/OpenGL rendering. Keep
  system libraries explicit; do not describe a single executable as independent
  of the OS, window system or graphics drivers.

## Acceptance

- [ ] Linux: build both native variants, check exports and run a bounded native
  window/render smoke test. The static archive links without a CUI shared library.
- [ ] Windows: independently build both variants and run the same basic window
  check. Verify calling convention, exported names and static-library inputs.
- [ ] Native distribution includes both linkage forms and all notices. Missing
  RID assets produce a precise build diagnostic rather than a runtime download.

Continue with the [managed binding](managed-binding.md). Packaging into consuming
FPAS projects belongs to [FPAS publishing](fpas-publishing.md).
