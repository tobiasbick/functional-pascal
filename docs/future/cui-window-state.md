# CUI window state

The FPAS GUI host exposes this API through `Options.Placement`,
`Session.Placement()` and the `Fpas.Ui.Gui.Windows` commands; see the
[window handbook page](../ui/app-chrome/window.md). The branch `window-state` in the
[source fork](https://github.com/tobiasbick/CUI/tree/window-state) adds window
state and placement to CUI's app layer. It is based on upstream `master` at
`6e6b353` and proposed in [issue #94](https://github.com/yappologistic/CUI/issues/94)
and [pull request #95](https://github.com/yappologistic/CUI/pull/95).
See the [desktop GUI application gaps](gui-app-gaps.md) for the FPAS side.

## Proposed API

- `cui_app_minimize`, `cui_app_restore` and `cui_app_window_state`. Restore
  undoes a maximize. A window minimized from maximized comes back maximized.
- `cui_app_set_position` places the content's top-left corner in desktop
  coordinates: pixels on Windows and X11, points on macOS.
- `cui_app_placement` returns the state and the normal size and place, also
  while the window is maximized or minimized. Its state is the state the
  window comes back in, so a window minimized from maximized saves as
  maximized. `cui_app_desc.placement` opens a window there. A maximized
  placement opens maximized and restores to the saved normal bounds. A place
  on no screen is left to the system.
- Wayland reports `has_position = false` and ignores positions.

Each backend notes the normal bounds where the system does not keep them:
X11, SDL2, GLFW and Cocoa. It notes them after a whole event batch, so a state
change that arrives with the resize is seen first. Win32 reads
`GetWindowPlacement`. The X11 backend sets `StaticGravity`, so positions name
the content rather than the window manager's frame.

## Verified

`tests/test_window_state.c` opens at a saved place, moves, maximizes,
minimizes, restores, and opens maximized. It checks state, content size and
placement after each step. On Windows and macOS an unperformed step fails; on
other systems it is reported as skipped.

| Environment | Result |
| --- | --- |
| Windows 11, MSVC, Win32, 150 % scale | All steps pass. Positions match `ClientToScreen` exactly. Full `ctest`: 22 of 22. |
| Windows 11, Win32, taskbar on the left (work area 72 pixels in), `9e2c556` | All steps pass. Placement while maximized and `cui_app_set_position` on a maximized window match `ClientToScreen` after the restore. |
| WSL Ubuntu 26.04, X11, Openbox on Xvfb | All steps pass with OpenGL and software rendering. `xwininfo` confirms the content positions. |
| WSL, SDL2 2.32.10 and GLFW 3.4, X11, Openbox on Xvfb | All steps pass. GLFW needs an unreachable Wayland socket to choose X11. |
| WSL, X11 on Xvfb without a window manager | State steps skip. A maximized placement is not claimed when no EWMH manager runs. |
| WSL, X11 with Openbox: full `ctest` and clang ASan/UBSan | 23 of 23; the sanitizer run of the app tests passes 3 of 3. |
| WSL, amalgamation with GCC and Clang | Builds as CI does, also after `tests/windows_sdk_words.h`. All documented programs build. |
| WSLg, X11 under Weston | Maximize and restore pass. Minimize is unsupported. Positions fail; see the open points. |
| WSLg, SDL2 and GLFW under Wayland | State steps pass and positions are reported as unavailable. Minimize is unsupported. GLFW keeps the maximized size after restore. |
| Debian Sid, KDE Plasma 6.7.4 Wayland at 130 %, SDL2 (sdl2-compat on SDL 3.4) | All steps pass in 3 runs. A minimized window cannot restore itself under xdg-shell, so those steps skip. |
| Debian Sid, KDE Plasma 6.7.4 Wayland, X11 backend through XWayland | All steps pass in 3 runs, including positions and minimize. `xwininfo` confirms the position. |
| Debian Sid, KDE Plasma 6.7.4 Wayland at 130 %, `2bafef3`: GLFW 3.5.1 | All steps pass in 3 runs, including restoring a window opened maximized. Minimize skips: GLFW never reports it on Wayland. |
| Debian Sid, KDE Plasma 6.7.4 Wayland, `2bafef3`: SDL2 and X11 through XWayland | Both pass, including the new placement checks. |
| Debian Sid, KWin 6.7.4 X11, `2bafef3`: X11 (OpenGL and software) and GLFW | All pass. The first frame reports 120,140, and a window minimized from maximized saves as maximized. |
| Debian Sid, KWin 6.7.4 X11, `9e2c556`: SDL2 (SDL 3.4.20), X11 and GLFW | All pass in 3 runs; SDL2 skips one step (open points). SDL 3 maps a window restored from minimized 28 pixels lower; CUI moves it back within 50 to 100 ms. |
| Debian Sid, KWin 6.7.4 X11, `4029237`: X11 (OpenGL and software), GLFW, SDL2 | All steps pass in 3 runs each. SDL2 skips one step; see the open points. |
| Debian Sid, KWin 6.7.4 X11, manual probe | Title-bar maximize, minimize, taskbar restore, hand moves and resizes, and reopening keep the normal bounds. `xwininfo` confirms the positions. Off-screen places are re-centered by KWin. |

## Open points

Record the CUI commit, session type and the test output verbatim. The KDE
VM keeps the builds, the diagnostic programs and the raw outputs of every run.

- [ ] SDL 3 on KWin X11 maps a window 28 pixels lower when it opens or is
  restored from minimized (reproduced without CUI). CUI moves it back, so it
  flickers for a few frames. Mention it in the pull request together with
  the dropped maximize in the next point; decide whether to report both to SDL.
- [ ] SDL2 on KWin X11: after a minimize from maximized, SDL 3.4 (also
  through sdl2-compat) restores the window to normal and ignores a following
  `SDL_MaximizeWindow`. Reproduced without CUI. The test reports a skip.
  Mention it in the pull request; decide whether to report it to SDL.
- [ ] X11: the on-screen check uses the bounding box of all monitors. With
  monitors of different sizes, a place in an empty corner of that box passes.
  KWin re-centers such windows. A second monitor was not available in the VM.
- [ ] GLFW on Wayland: a minimized window blocks in `eglSwapBuffers` because
  WSLg's compositor sends no frame callbacks. This is an existing GLFW
  backend defect, not part of this change. It did not occur on KWin, where
  GLFW never reports the window as minimized.
- [ ] X11 under XWayland: CUI rounds the 130 % desktop scale to 125 %
  (`Xft.dpi` 124). This is the existing X11 scale handling, not part of this
  change; decide separately whether to report it.
- [ ] WSLg's Weston ignores `StaticGravity`: X11 positions land 6 and 27
  pixels off, the size of its frame. KWin and Openbox honor it. Mention it
  in the pull request.
- [ ] Windows: placement across monitors with different scales. Only one
  monitor was available; a taskbar on the left is verified.
- [ ] macOS: the Cocoa code was written without a Mac. Upstream CI builds it
  and runs the smoke tests; window behavior on macOS is untested, as the pull
  request says.
- [ ] Follow the upstream review of pull request #95. Its head `f259305`
  contains the fixes for the first CodeRabbit review and is merged into the
  vendored `fpas-cui` branch. Two findings stay unchanged with a reply in the
  pull request: X11 reports no place for a window opened maximized until it
  is restored, and the SDL2 backend keeps window units under `CUI_SCALE`.
  Update the FPAS binding if the review changes the API.
