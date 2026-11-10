# GUI distribution Windows acceptance

The [handbook](../../pascal/libraries/gui.md), [host reference](../../../src/Fpas.Gui/README.md),
[publishing reference](../../build/publishing.md) and
[acceptance map](../../../tests/coverage.md#native-gui) describe the implemented
application and its checks. Phase 8 still requires native Windows x64 evidence.

## Windows desktop

- [ ] Build Debug and Release using the Windows native toolchain. Run compiler,
  SDK/documentation, GUI/shared-widget and TUI regressions for those configurations.
- [ ] Set `FPAS_TEST_GUI_AOT=1` and run the complete Fpas.Gui.Cui.Tests suite
  with software and OpenGL rendering. Retain screenshots and logs using
  `FPAS_GUI_TEST_ARTIFACTS` under `.temp-data`.
- [ ] Run both publishing matrices: the signed binding fixture and the public
  package-only gallery. Check all five modes, RID selection, six license notices,
  executable-only single-file/NativeAOT startup, extraction and dumpbin imports.
  Start from another directory with runtime/library search overrides removed.
- [ ] Exercise real Win32 pointer and keyboard delivery, Unicode editing,
  modal Escape/focus restoration, resize with theme/model state preserved,
  and native window close. The X11/XTest case is Linux-specific and is skipped
  on Windows; injected CUI input does not replace this OS event-path check.
- [ ] Verify the packaged single-file and NativeAOT gallery with native resize,
  theme changes and close, including work/cancellation pending during cleanup.
  Use the [phase 6 host checks](gui-host.md) for live-resize callback and shutdown
  ownership requirements.
- [ ] Confirm resource retirement and repeated owner-thread teardown using
  native texture accounting and retained editor/image counts. Investigate
  unexplained growth with native diagnostics rather than a working-set threshold.
- [ ] Close the [phase 6](gui-host.md) and [phase 7](tui-alignment.md) Windows
  checks, resolve findings and remove this folder when no acceptance work remains.

Linux OS input uses real X server events through XTest. Other input fixtures use
direct CUI injection. Keep that distinction in any platform evidence. Screenshots
and temporary deliveries belong under ignored `.temp-data`; no cloud timing or
binary-size observations belong in permanent benchmark history.
