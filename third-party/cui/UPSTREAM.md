# CUI source dependency

| Property | Value |
| --- | --- |
| Repository | <https://github.com/yappologistic/CUI> |
| Source fork | <https://github.com/tobiasbick/CUI> |
| Revision | Upstream `master` after [v1.4.0](https://github.com/yappologistic/CUI/releases/tag/v1.4.0) |
| Commit | [`e0124eaef2a41f07c7d6f22aed9af48314f1b3f3`](https://github.com/yappologistic/CUI/commit/e0124eaef2a41f07c7d6f22aed9af48314f1b3f3) |
| Build inputs | [`cui.c`](upstream/cui.c) and [`cui.h`](upstream/cui.h) |
| Local source modifications | None |

The amalgamated C source and public header are the unchanged output of
`python3 tools/amalgamate.py` on Linux from the upstream tree at the pinned commit.
This upstream snapshot includes the fixes from
[PR #102](https://github.com/yappologistic/CUI/pull/102),
[PR #105](https://github.com/yappologistic/CUI/pull/105),
[PR #107](https://github.com/yappologistic/CUI/pull/107),
[PR #108](https://github.com/yappologistic/CUI/pull/108) and
[PR #110](https://github.com/yappologistic/CUI/pull/110), which the maintainer
applied to master by hand. No combined fork branch is needed.

| Path under `upstream/` | SHA-256 |
| --- | --- |
| `cui.c` | `1ac237a2f9d51afcb6023ebc63c1a072978f51413cc2870a51da3a1dbbacbfc0` |
| `cui.h` | `6521fb914d75e2d67841ebd5385206edfcfa6a125e6def11a9c029e62ac69fae` |

Project-owned integration belongs under `src/Fpas.Ui.Gui.Cui/`, with the C bridge
and CMake configuration in its `native/` directory. Keep those files separate
from the unmodified upstream sources here.

## Changes since `56c9bc2`

The five upstream commits change the Win32 OpenGL host and add an explicit
SDL2 Wayland platform. Windows temporarily suspends vertical sync during
maximize, minimize and restore, then resumes the configured swap interval.
FPAS builds the native X11 host on Linux. The SDL2 build option is relevant
to embedders that select that backend. The public header, widget inventory,
theme palettes and license inputs are unchanged.

The FPAS adapter now exposes [focusable canvases and keyboard input](../../docs/ui/gui/text-and-display/canvas.md),
pane focus scopes and [RGBA region updates](../../docs/ui/gui/text-and-display/image.md).
These use functions introduced in the preceding snapshot.

The generated pair was compared byte for byte with the vendored files. Under
Linux/X11, 55 native tests and 279 managed binding tests passed, including
platform keyboard input and pixel comparisons between partial and full uploads
with the software and OpenGL renderers. Five Windows binding tests were skipped.
The FPAS gallery and its drawing contract passed with both renderers. The Windows x64 DLL and [WGL regression executable](../../src/Fpas.Ui.Gui.Cui/native/tests/window_transition.c)
were cross-compiled and linked on Linux. Running that test requires a Windows
desktop and a WGL driver with swap control.

[Window transition fix](https://github.com/yappologistic/CUI/commit/0f9c2d1) ·
[SDL2 Wayland backend](https://github.com/yappologistic/CUI/commit/cadb1d9)

## Verification history for `56c9bc2`

The copied pair matches the generated files byte for byte. License inputs,
fonts, `third_party/` and `tools/amalgamate.py` are unchanged from the previous
pin. The theme tokens and palettes are unchanged; theme fades now advance with
the transition step instead of the frame time. The Direct3D 12, Vulkan and
Metal renderers are built only with `CUI_D3D12`, `CUI_VULKAN` or `CUI_METAL` and
are not part of the amalgamation.

The fixes for CUI-033, CUI-044, CUI-045 and CUI-048 are in this snapshot:

- The planning adapters no longer guard drags. The native planning regression
  passes read-only, disabled-input and removed-item interruptions through CUI
  itself.
- The PNG capture bridge no longer invalidates the software buffer address
  before each screenshot. Upstream `cui_test_app_screenshot` passes on Windows
  and under WSL.
- Upstream checks cover the first hover frame after an idle pause and wheel
  areas in translated nodes. A local check also takes the wheel at a dragged
  dialog's moved canvas.

CUI-043 is fixed upstream as well; its KWin and Mesa DRI3 reproduction has not
been rerun. The chart regression still reproduces CUI-019. GCC 15 reports a new
array-bounds warning in `cui.c`, tracked as CUI-049 and suppressed for GCC in
the native CMake configuration.

Upstream checks at `56c9bc2`: `cui_tests` and ctest passed all 26 tests on
Windows with MSVC. Under WSL, the clang ASan/UBSan build passed 27 of 28 tests;
`cui_test_window_state` fails there exactly as it does at `71c383f`.
`cui_test_x11_idle` was not built because the WSL image lacks `xcb-present`.

The native integration suite passed all 57 tests on Windows x64 with MSVC and
all 55 tests on Linux x64 under WSL with GCC 15.2 and Xvfb, including the
planning checks against the adapters and against CUI directly.

The managed binding suite passed 275 tests on Windows with five platform or
publishing skips, and 273 tests on Linux under WSL with GCC 15.2 and Xvfb, with
seven skips. The Linux binding build passed with `-warnaserror` from a clean
tree. The Windows build reports the existing upstream Gantt shadowing warning
C4456.

Publishing, NativeAOT, macOS, and SDL2/GLFW runtime checks were not run for
this update.

See [CUI upstream bugs](../../docs/future/cui-upstream-bugs.md) for remaining defects.

## Changes between `71c383f` and `56c9bc2`

The 28 commits after `71c383f` add public API without changing existing
declarations or the widget inventory:

- Input capture and ordered dispatch (`cui_set_input_hook`,
  `cui_dispatch_input`), focusable canvases (`cui_canvas_ex`,
  `cui_request_component_focus`), focus scopes for pane-scoped Tab navigation,
  `cui_set_cursor`, `cui_frame_alloc`, `cui_frame_strdup` and
  `cui_image_update`.
- Tab and Shift+Tab stay within a modal and its open popups. The adapters no
  longer move popover focus entries into the parent layer.
- Planning drags end when the item list shrinks, input is disabled or
  `read_only` changes. Repeated software screenshots render the full frame.
  Transitions that start after an idle pause take one frame step. Wheel areas
  and zoom anchors follow their node's transform. Idle X11 windows ignore
  private Present notifications.
- Internally, the modal layer is a `cui_modal_layer` struct and the focus order
  holds `cui__focus_entry` records. The command palette, shortcut ownership,
  host frame start and table adapters use the new fields.

## License inventory

The paths in the middle column refer to the pinned upstream repository.
Standalone notices accompany binary distributions.

| Local notice | Upstream source | Terms |
| --- | --- | --- |
| [LICENSE-CUI.txt](LICENSE-CUI.txt) | `LICENSE` | MIT |
| [LICENSE-stb.txt](LICENSE-stb.txt) | License blocks in `third_party/stb_image.h`, `stb_image_write.h`, `stb_rect_pack.h`, `stb_truetype.h` | MIT or public domain, at the recipient's option |
| [LICENSE-Unicode.txt](LICENSE-Unicode.txt) | `third_party/LICENSE-Unicode.txt` | Unicode License V3 |
| [LICENSE-Inter.txt](LICENSE-Inter.txt) | `assets/fonts/LICENSE-Inter.txt` | SIL Open Font License 1.1 |
| [LICENSE-Geist.txt](LICENSE-Geist.txt) | `assets/fonts/LICENSE-Geist.txt` | SIL Open Font License 1.1 |
| [LICENSE-Lucide.txt](LICENSE-Lucide.txt) | `assets/fonts/LICENSE-Lucide.txt` | ISC, including Feather-derived icons under MIT |

The CUI, Unicode and font/icon notice files are copied unchanged. The stb notice
contains the dual-license text from the stb code in `cui.c`.

Keep these notices when redistributing the dependency, including native,
single-file and NativeAOT application distributions.

## Updating the dependency

1. Choose an upstream commit. Resolve the tag when updating to a release.
2. Review source and license changes against the previous pin. Generate the
   pair with `python3 tools/amalgamate.py` on Linux or WSL so it uses LF endings.
3. Copy `cui.c` and `cui.h` from `build/amalgamation/` and verify that their
   SHA-256 digests match the generated files. For a release, compare the assets.
4. Refresh the revision, commit and digests. Check whether the new upstream
   tree changes the license inventory or notices.
5. Check the semantic palettes exported by CUI's theme builder against
   [`ThemePalettes.fpas`](../../src/Fpas.Ui/Internal/Theming/ThemePalettes.fpas) and the
   neutral-light defaults in [`Themes.fpas`](../../src/Fpas.Ui/Widgets/Theming/Themes.fpas).
   These values supply both hosts without adding native dependencies to TUI.
   Keep token and accent order aligned with the header; the native bridge
   checks the token count at compile time.
6. Run the native, binding and FPAS-consumer checks, including shared-theme
   and layout parity in `GuiHostTests` and the terminal CUI contract fixture.
