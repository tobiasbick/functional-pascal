# Motion

The gallery shows text effects and animations alongside compact controls.
The text effects respect the host's reduced-motion setting. The application
owns the text, selection and playback values that drive these examples.
These widgets require the GUI host.

| Widget | When to use it |
| --- | --- |
| [Shimmer text](shimmer-text.md) | Show a moving highlight on a waiting message. |
| [Typewriter](typewriter.md) | Reveal streaming text and optionally report completion. |
| [Text reveal](text-reveal.md) | Fade and raise words into place. |
| [Rotating words](rotating-words.md) | Cycle through short words or phrases. |
| [Rolling number](rolling-number.md) | Animate changes in a formatted number. |
| [Gradient text](gradient-text.md) | Paint a heading with a static or animated gradient. |
| [Spotlight card](spotlight-card.md) | Draw a card with pointer lighting and an optional beam. |
| [Loader](loader.md) | Indicate pending work with one of five animations. |
| [Dock](dock.md) | Provide a magnifying icon row with status dots and badges. |
| [Control tile](control-tile.md) | Toggle a setting with a large icon and status line. |
| [Now playing](now-playing.md) | Display track artwork, transport and progress. |
| [Waveform](waveform.md) | Draw audio peaks with an optional seek position. |
| [Tab bar](tab-bar.md) | Switch views using icons and badges in compact layouts. |

## See also

- [UI handbook](../README.md)
- [Compact controls in the gallery](../../../../examples/ui/gui/gallery/Pages/Motion/Controls.fpas)
- [Shared text options](../../../../src/Fpas.Ui/Widgets/Motion/Motion.fpas)
- [Audio](../audio/README.md)
- [Motion in the gallery](../../../../examples/ui/gui/gallery/Pages/Motion/View.fpas)

## Source files

- `examples/ui/gui/gallery/Pages/Motion/`
  - [Clock.fpas](../../../../examples/ui/gui/gallery/Pages/Motion/Clock.fpas)
  - [State.fpas](../../../../examples/ui/gui/gallery/Pages/Motion/State.fpas)
  - [Update.fpas](../../../../examples/ui/gui/gallery/Pages/Motion/Update.fpas)
- `src/Fpas.Ui.Gui.Cui/Widgets/Motion/`
  - [CuiMotionText.Types.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Motion/CuiMotionText.Types.cs)
  - [CuiMotionText.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Motion/CuiMotionText.cs)
- `src/Fpas.Ui.Gui.Cui/native/tests/`
  - [motion.c](../../../../src/Fpas.Ui.Gui.Cui/native/tests/motion.c)
- `src/Fpas.Ui.Gui.Cui/native/widgets/motion/`
  - [motion_text.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/motion/motion_text.c)
  - [motion_text.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/motion/motion_text.h)
