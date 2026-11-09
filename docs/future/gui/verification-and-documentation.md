# Verification and documentation

Close the implementation with real FPAS consumers, packaged assets and platform
evidence. Per-stage checks belong to their owning files; this stage verifies the
whole application path and makes it usable outside the repository.

## Integration matrix

- [ ] Publish an application built on the public FPAS GUI host on Linux x64
  and Windows x64 in all five modes. Verify native asset selection, software
  and OpenGL rendering, windowed startup and UI-thread teardown.
- [ ] Use packaged SDK/library assets for end-to-end tests. Launch from another
  working directory without source/build-tree paths or library search overrides.
- [ ] For NativeAOT, move only the executable to an empty directory, remove .NET
  search overrides and confirm no own native-library extraction or CUI dependency.
  Inspect the native result as well as its exit code.
- [ ] Cover button, checkbox, UTF-8 editing, focus, modal interaction, scrolling,
  background completion, cancellation and shutdown through the public FPAS host.
- [ ] Use the [FPAS GUI gallery](gui-host.md#widget-gallery-example) as a real
  application consumer. Check bounded navigation/interaction scenarios and
  screenshots under Xvfb on Linux and on Windows, with both renderers.
- [ ] Publish and launch the gallery in NativeAOT and managed single-file modes
  using packaged assets; include it in checks for resize/theme state preservation.
- [ ] Distinguish injected CUI input tests from tests of OS input delivery.
  Add a bounded OS event-path check where platform automation is available;
  record unsupported environments precisely rather than claiming verification.
- [ ] Check repeated session/resource lifetimes with stable owned-resource counts
  and explicit disposal assertions. Use native diagnostics for unexplained growth;
  process working set alone does not establish a leak.
- [ ] Keep snapshots/screenshots and temporary publish outputs in ignored
  `.temp-data` locations. Do not turn cloud timing or executable-size observations
  into permanent benchmark/history entries.

## User and agent documentation

- [ ] Add a concise GUI handbook entry and runnable FPAS examples using the
  canonical MVU/widget API. Explain startup, model/messages, effects, input and
  deterministic cleanup with one normal authoring path.
- [ ] Link the GUI gallery from the GUI/library guidance and example index,
  alongside the TUI gallery. Document its pages and supported widget variants.
- [ ] Document common widgets, the TUI support table, backend extensions, layout
  units, editor positions and which views/models are portable.
- [ ] Describe normal publishing versus NativeAOT static linking. State source
  build prerequisites, consumer asset delivery and OS/window/driver dependencies.
  Do not ask GUI consumers to download/build CUI manually.
- [ ] Record actual platform limits: Windows/Linux x64 support, software/OpenGL,
  direct Wayland limitations, accessibility and complex-text/IME constraints.
  Do not promise capabilities that the adapter or CUI does not supply.
- [ ] Update the agent reference with a focused route to the implemented GUI/MVU
  guidance. Keep native internals and long build procedures in their owning docs.
- [ ] Update library, SDK/distribution and test references where their contracts
  change. Verify links and compile/run the new FPAS documentation examples.

## Closure

- [ ] Resolve implementation findings and complete the affected compiler/SDK,
  GUI/MVU/shared-widget and TUI regression checks. Mark unavailable platform
  checks as open rather than treating cloud-only success as completion.
- [ ] Ensure the owning references describe current behavior. Remove completed
  work from these plans and delete the folder once no implementation work remains.
