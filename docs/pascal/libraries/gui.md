# Desktop GUI with shared MVU widgets

Use `Fpas.Ui.Gui` for a native desktop window. Its model, messages, update function
and declarative view use the same [MVU](../../../src/Fpas.Ui.Mvu/README.md) and
[widget contracts](../../../src/Fpas.Ui/README.md) as the
[terminal host](../../../src/Fpas.Ui.Tui/README.md). Terminal cell styling,
shortcuts and exact viewport access are explicit terminal capabilities.

## Project and examples

Reference the `Fpas.Ui.Gui` package from the feed containing the FPAS libraries for
your runtime identifier. It brings `Fpas.Ui.Mvu`, `Fpas.Ui` and `Fpas.Ui.Gui.Cui`
transitively. Use the same release of the libraries and SDK.

```xml
<Project Sdk="Fpas.NET.Sdk/0.1.0-dev">
  <PropertyGroup>
    <TargetFramework>net10.0</TargetFramework>
    <OutputType>Exe</OutputType>
  </PropertyGroup>
  <ItemGroup>
    <PackageReference Include="Fpas.Ui.Gui" Version="0.1.0-dev" />
  </ItemGroup>
</Project>
```

The [gallery](../../../examples/ui/gui/gallery/README.md) demonstrates the
accepted widgets, layout, theme switching and responsive navigation.
Its checked-in project uses source references for repository development;
application packages deliver the managed libraries, native assets and notices.
Users of those packages do not download CUI or build its C sources.

[Smart inputs](../../../src/Fpas.Ui/README.md#smart-inputs) provides native
inline editing, numeric scrubbing, money fields and shortcut recording/labels.
Each keeps rendering and interaction in CUI and emits model-owned proposals.

Native [accordions and expandable cards](../../../src/Fpas.Ui/README.md#accordion-and-expandable-card)
use model-owned expansion. [Expandable text](../../../src/Fpas.Ui/README.md#expandable-text)
keeps its local expansion in CUI. These widgets require the GUI host.
Native [alerts, progress bars and spinners](../../../src/Fpas.Ui/README.md#alerts-progress-and-spinners)
also require the GUI host. CUI owns alert dismissal and feedback animation.
The [capacity meter](../../../src/Fpas.Ui/README.md#meter) uses CUI's numeric display,
track and warning colors.
The [progress ring](../../../src/Fpas.Ui/README.md#progress-ring) delegates its
centered percentage, optional caption and interpolation to CUI.
The [timeline item](../../../src/Fpas.Ui/README.md#timeline-item) supplies native
activity entries with literal time captions, descriptions, icons and connectors.
The [skeleton](../../../src/Fpas.Ui/README.md#skeleton) uses native loading shimmer
and CUI's reduced-motion behavior, with fill, fixed or percentage width.
The [guided tour](../../../src/Fpas.Ui/README.md#guided-tour) adds native
spotlights and explanation cards to model-selected controls.

## Model, update and view

Keep application state in an immutable record. Define a message variant for user
actions and completed work. Update has type
`function(Model, Msg): Mvu.Transition<Model, Msg>` and returns the next model
plus commands. View has type
`function(Model, Layout.Viewport): Elements.Element<Msg>` and builds widgets
from that model using the `Fpas.Ui` widget units.

Control IDs stay stable and unique; item keys are local to their collection.
Theme selections, navigation, sheet visibility and action feedback belong to
the model. Rejecting a proposal leaves the model value authoritative.

Start the owner loop with
`Gui.Run<Model, Msg>(new Model(), Update, View, new Gui.Options())`.
Options select title, logical width/height, renderer and mailbox capacity.
The host validates the whole active view before drawing. Explicit GUI sizing
uses portable `Layout.Measure.Logical`; the terminal adapter translates those
distances to cells. Layout follows the [shared CUI rules](../../../src/Fpas.Ui/README.md#layout-and-style).
GUI additionally accepts `Layout.Extent.Percent` for main-axis sizing and button
or skeleton width. CUI resolves the percentage against the parent's inner extent.
The [graphical layout extensions](../../../src/Fpas.Ui/README.md#native-graphical-layout)
include overlapping children, independent dimensions, per-edge padding,
LTR/RTL flow, native spacing and horizontal scrolling.
Both hosts accept `Fpas.Ui.Themes.Theme`, for example `Themes.Dark(Themes.Accent.Blue)`,
and the same `ThemeFor(Model)` callback. Set a fixed theme with `Gui.Options.Theme`.
Palette customization and terminal approximations are described in
[shared themes](../../../src/Fpas.Ui/README.md#shared-themes).

For larger applications, use feature slices and cached subviews as described in
the [MVU scaling guidance](../../../src/Fpas.Ui.Mvu/README.md#larger-applications).
Subview caching does not skip all host processing or virtualize large collections.

## Work and cleanup

Use shared `Mvu.StartBackground`, `CancelBackground`, `Focus` and `Exit` commands.
Background work receives a typed Post callback and cancellation token. Update
and View stay on the creating UI thread. Do not block them while waiting for a
worker; post its result and let the next owner turn apply it.

For a manually driven application, own `Gui.Start` with `using const` and call
Step on its creating thread until it returns false. Post can run on workers and
reports Queued, Full or Closed. Close cancels work, rejects late results and
keeps pumping the frozen view until cleanup finishes. Disposal joins cleanup and
releases native resources on the owner. Step, Snapshot and disposal must not be
reentered from callbacks.

Snapshot describes the last successful frame, including control bounds, values,
focus and theme. `Session.CapturePng(Path)`
saves the presented native frame outside Step on the owner thread. It requires
an existing destination directory and a successful presentation; capture failure
does not close the session. PNG dimensions are physical pixels and can differ
from logical dimensions with desktop scaling.

## Publishing and platform limits

Use the normal [five publish modes](../../build/publishing.md). Framework-dependent,
apphost and self-contained applications carry the shared native library beside
the application. Single-file bundles it for .NET extraction. NativeAOT statically
links CUI so the executable needs no separate CUI library or extraction. All modes
deliver license notices. NativeAOT still requires the .NET platform's native
compiler/linker and development libraries when publishing.

The initial targets are Windows x64 and Linux x64. Linux uses X11, including
XWayland where available, and needs an accessible display. Headless validation
uses Xvfb. CUI provides software and OpenGL renderers; Auto can fall back to
software. Graphics drivers and OS window libraries remain runtime dependencies.
See [native prerequisites](../../../src/Fpas.Ui.Gui.Cui/native/README.md#prerequisites)
for source builds and [host behavior](../../../src/Fpas.Ui.Gui/README.md) for details.

Direct Wayland, other RIDs, screen-reader accessibility and complete platform
IME/complex-script behavior are outside the supported contract. Unicode text
does not imply full bidirectional shaping or all desktop input methods.
One UI thread can own one OpenGL CUI window; Auto
uses software for additional windows on that thread.

## See also

- [Shared widgets and capability profiles](../../../src/Fpas.Ui/README.md)
- [GUI host lifecycle and inspection](../../../src/Fpas.Ui.Gui/README.md)
- [GUI gallery](../../../examples/ui/gui/gallery/README.md)
- [Native assets and local distribution feed](../../build/distribution.md#native-cui-assets)
- [Terminal widget gallery](../../../examples/ui/tui/gallery/README.md)

## Charts

The [shared chart API](../../../src/Fpas.Ui/README.md#charts) exposes native
line/area/dither, grouped/stacked bars, donuts, gauges and pattern meters.
CUI owns their layout, palettes, animation and interaction. The
[Charts page](../../../examples/ui/gui/gallery/Pages/Charts/View.fpas)
reproduces the original eight examples, including the storage meter's
Grayscale switch and native hover/keyboard readout.

The [visualization API](../../../src/Fpas.Ui/README.md#visualizations) adds
matrix/calendar heatmaps, treemap, radar, funnel, split bar and waffle.
Their [More charts examples](../../../examples/ui/gui/gallery/Pages/MoreCharts/View.fpas)
use native CUI layout and interaction. Optional click callbacks report input
indices, matrix positions or actual calendar dates; treemap selection is
model-owned.

[Flows, ranks and instruments](../../../src/Fpas.Ui/README.md#flows-ranks-and-instruments)
add Sankey, animated bar lists, activity rings, scatter/bubble charts, radial
gauges and candlestick/OHLC charts. CUI owns their rendering and interaction,
including candlestick zoom, panning, keyboard inspection and volume. The GUI
gallery's More charts page reproduces every original example.
