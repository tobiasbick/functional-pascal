# CUI source dependency

| Property | Value |
| --- | --- |
| Repository | <https://github.com/yappologistic/CUI> |
| Source fork | <https://github.com/tobiasbick/CUI> |
| Revision | Upstream `master` after [v1.4.0](https://github.com/yappologistic/CUI/releases/tag/v1.4.0), mirrored in the source fork |
| Commit | [`56c9bc2c985b21bacb79d34c842980071c52800a`](https://github.com/yappologistic/CUI/commit/56c9bc2c985b21bacb79d34c842980071c52800a) |
| Build inputs | [`cui.c`](upstream/cui.c) and [`cui.h`](upstream/cui.h) |
| Local source modifications | None |

The amalgamated C source and public header are the unchanged output of
`python3 tools/amalgamate.py` on Linux in the source fork at the pinned commit.
This upstream snapshot includes the fixes from
[PR #102](https://github.com/yappologistic/CUI/pull/102),
[PR #105](https://github.com/yappologistic/CUI/pull/105),
[PR #107](https://github.com/yappologistic/CUI/pull/107),
[PR #108](https://github.com/yappologistic/CUI/pull/108) and
[PR #110](https://github.com/yappologistic/CUI/pull/110), which the maintainer
applied to master by hand. No combined fork branch is needed.

| Path under `upstream/` | SHA-256 |
| --- | --- |
| `cui.c` | `2d1190191f98f969cdcf5e986fed1e1dbd90ff78c48a0077cf7f03e39bb4dd87` |
| `cui.h` | `6521fb914d75e2d67841ebd5385206edfcfa6a125e6def11a9c029e62ac69fae` |

Project-owned integration belongs under `src/Fpas.Ui.Gui.Cui/`, with the C bridge
and CMake configuration in its `native/` directory. Keep those files separate
from the unmodified upstream sources here.

## Snapshot verification

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

## Changes from the previous snapshot

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

The FPAS bindings do not expose the new public API yet.

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
