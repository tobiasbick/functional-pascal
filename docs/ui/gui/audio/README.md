# Audio

The gallery combines a synth and a six-channel mixer. Knobs and faders send
proposed values to the application. Level meters draw the peaks supplied by
the model. These widgets require the GUI host.

| Widget | When to use it |
| --- | --- |
| [Knob](knob.md) | Adjust frequency, attack, resonance or pan with a rotary control. |
| [Fader](fader.md) | Set a channel gain and optionally show signal levels beside it. |
| [Level meter](level-meter.md) | Display per-channel peaks with decay, peak hold and clip indicators. |

Specify a gain control's range in the units the user edits, such as dB.
Supply meter peaks as linear amplitudes, where 1.0 represents 0 dBFS.

## Audio gallery page

The [Audio view](../../../../examples/ui/gui/gallery/Pages/Audio/View.fpas)
places the tour button above a wrapping row. The synth card contains four
knobs and a horizontal stereo master meter. The mixer card wraps six faders
with stereo meters. The tour targets the Cutoff knob, the master meter and
the Bass fader through stable control IDs.

## Signal clock

The original CUI gallery advances the synthetic signal once per frame. FPAS
advances the signal through a background effect while Audio is visible. The
gallery keeps the control values and signal phase when the page closes.

## See also

- [UI handbook](../README.md)
- [Guided tour](../overlays/guided-tour.md)
- [Audio in the gallery](../../../../examples/ui/gui/gallery/Pages/Audio/View.fpas)

## Source files

- `examples/ui/gui/gallery/Pages/Audio/`
  - [Clock.fpas](../../../../examples/ui/gui/gallery/Pages/Audio/Clock.fpas)
  - [State.fpas](../../../../examples/ui/gui/gallery/Pages/Audio/State.fpas)
  - [Update.fpas](../../../../examples/ui/gui/gallery/Pages/Audio/Update.fpas)
- `src/Fpas.Ui.Gui.Cui/native/widgets/audio/`
  - [control_well.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/audio/control_well.c)
- `tests/Fixtures/gui-host/`
  - [AudioContract.fpas](../../../../tests/Fixtures/gui-host/AudioContract.fpas)
