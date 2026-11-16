# CUI window state follow-ups

- [Window API](../ui/app-chrome/window.md) · [FPAS application gaps](gui-app-gaps.md)
- [Issue #94](https://github.com/yappologistic/CUI/issues/94) · [PR #95](https://github.com/yappologistic/CUI/pull/95)
- Fixes: [`a33a7ec`](https://github.com/yappologistic/CUI/commit/a33a7ec), [`1e172ba`](https://github.com/yappologistic/CUI/commit/1e172ba), [`ee913d1`](https://github.com/yappologistic/CUI/commit/ee913d1)
- Follow-ups: [`89f7a11`](https://github.com/yappologistic/CUI/commit/89f7a11), [`961f096`](https://github.com/yappologistic/CUI/commit/961f096), [`5848bf8`](https://github.com/yappologistic/CUI/commit/5848bf8)
- [Upstream confirmation](https://github.com/yappologistic/CUI/pull/95#issuecomment-6907824603)
- Pinned snapshot: [`a5c9ef5`](https://github.com/yappologistic/CUI/commit/a5c9ef5efc39b3fc783758362f6b8f9f130f5883)

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
- [ ] macOS: local interactive verification remains unavailable. The maintainer
  reports the window-state test passing in macOS CI in
  [PR #95](https://github.com/yappologistic/CUI/pull/95#issuecomment-6907824603).
- [ ] SDL2 with software rendering and `CUI_SCALE`: placement dimensions use
  window units while `cui_app_size` divides by the overridden scale. Review
  creation, size-setting and placement together before changing the units;
  converting only saved placement would change the reopened window size.
  See the [deferred review finding](https://github.com/yappologistic/CUI/pull/95#discussion_r4579776699).
