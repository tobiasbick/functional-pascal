# CUI window state follow-ups

- [Window API](../ui/gui/app-chrome/window.md) · [FPAS application gaps](gui-app-gaps.md)
- [Issue #94](https://github.com/yappologistic/CUI/issues/94) · [PR #95](https://github.com/yappologistic/CUI/pull/95)
- Fixes: [`a33a7ec`](https://github.com/yappologistic/CUI/commit/a33a7ec), [`1e172ba`](https://github.com/yappologistic/CUI/commit/1e172ba), [`ee913d1`](https://github.com/yappologistic/CUI/commit/ee913d1)
- Follow-ups: [`89f7a11`](https://github.com/yappologistic/CUI/commit/89f7a11), [`961f096`](https://github.com/yappologistic/CUI/commit/961f096), [`5848bf8`](https://github.com/yappologistic/CUI/commit/5848bf8)
- [Upstream confirmation](https://github.com/yappologistic/CUI/pull/95#issuecomment-6907824603)
- [Vendored CUI revision](../../third-party/cui/UPSTREAM.md)

## Open points

Record the CUI commit, session type and the test output verbatim. The KDE
VM keeps the builds, the diagnostic programs and the raw outputs of every run.

- [ ] Decide whether to report the two SDL 3 behavior differences on KWin X11
  documented in [PR #95](https://github.com/yappologistic/CUI/pull/95). These
  reproduce with SDL alone. A window opens or restores from minimized 28 pixels
  lower, and a window minimized from maximized restores to normal and ignores
  a following `SDL_MaximizeWindow`. CUI corrects the position after a few frames.
  The window-state test skips the dropped maximize step.
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
- [ ] macOS: local interactive verification remains unavailable. The maintainer
  reports the window-state test passing in macOS CI in
  [PR #95](https://github.com/yappologistic/CUI/pull/95#issuecomment-6907824603).
- [ ] SDL2 with software rendering and `CUI_SCALE`: placement dimensions use
  window units while `cui_app_size` divides by the overridden scale. Review
  creation, size-setting and placement together before changing the units;
  converting only saved placement would change the reopened window size.
  See the [deferred review finding](https://github.com/yappologistic/CUI/pull/95#discussion_r4579776699).
