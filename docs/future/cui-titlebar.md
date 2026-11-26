# Optional CUI title bar

CUI accepted an opt-in, app-drawn title bar in
[#130](https://github.com/yappologistic/CUI/issues/130), which also answers
[#21](https://github.com/yappologistic/CUI/issues/21). It arrives in two pull
requests. The vendored revision contains PR1; FPAS integration starts once it
also contains PR2.
See the [desktop GUI application gaps](gui-app-gaps.md) for priorities.

## Upstream pull requests

1. **PR1, [#134](https://github.com/yappologistic/CUI/pull/134), on upstream
   `master`.** GitHub still lists the pull request as open. It adds
   `cui_app_desc.custom_title_bar` with the Win32 frame, the optional
   `cui_platform_iface.window_frame` and `.window_button` hooks and
   `cui_title_bar_hit_test`, with a Win32 test. Until PR2 lands, a window with
   `custom_title_bar` has no caption and cannot be dragged.
2. **PR2, not opened.** The `cui_title_bar` chrome component in its own
   component file, with its documentation page and images. Base it on upstream
   `master`, where `cui_title_bar_hit_test` lives in `cui_core.c`. Run
   `tools/check_pr.py` and `tools/check_pr_description.py` from upstream's
   `AGENTS.md` before opening it, and ask the user before publishing it.

Neither PR covers the X11 frame, the runtime toggle
`cui_app_set_custom_title_bar`, `cui_split`'s `.fixed_first` or transparency
effects such as Mica. The fork branch `codex/cui-titlebar-testing` keeps the
prototype for these.

## Upstream requirements

- Off by default: an app without `custom_title_bar` keeps today's window on
  every backend.
- Native Windows behavior: resize edges, the shadow, dragging, double-click to
  maximize and Snap layouts on the maximize button (`HTMAXBUTTON`).
- Where a backend keeps the system title bar, `cui_title_bar` draws the same
  bar without window buttons, so one layout works everywhere.
- The core stays decoupled: platform pieces go through the optional hooks.

## FPAS integration

- Expose the title bar through the binding and the GUI host once the vendored
  revision contains it. Keep the system frame as the default; failed activation
  leaves it intact and the bar renders as an ordinary bar.
- Keep macOS on the system frame until it can be implemented and validated on
  macOS. Validate frame behavior in native desktop sessions; CI compilation and
  WASM screenshots do not cover native window decorations.

## References

- [Windows title-bar customization](https://learn.microsoft.com/en-us/windows/apps/develop/title-bar)
- [Win32 custom window frames](https://learn.microsoft.com/en-us/windows/win32/dwm/customframe)
- [CUI contribution guidelines](https://github.com/yappologistic/CUI/blob/master/docs/CONTRIBUTING.md)
