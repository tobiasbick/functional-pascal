# Desktop GUI with shared MVU widgets

Use `Fpas.Ui.Gui` for a native desktop window. Its model, messages, update function
and declarative view use the same [MVU](../../../src/Fpas.Ui.Mvu/README.md) and
[widget contracts](../../../src/Fpas.Ui/README.md) as the
[terminal host](../../../src/Fpas.Ui.Tui/README.md). Terminal cell extensions and
GUI images/canvases are explicit capabilities; a portable view uses neither.

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

The [counter program](../../../examples/ui/gui/counter/main.fpas) is a complete
example of controlled inputs, background work and a modal. The
[gallery](../../../examples/ui/gui/gallery/README.md) demonstrates every supported
widget, layout, image/canvas, theme switching and responsive navigation.
Their checked-in projects use source references for repository development;
application packages deliver the managed libraries, native assets and notices.
Users of those packages do not download CUI or build its C sources.

## Model, update and view

Keep application state in an immutable record. Define a message union for user
actions and completed work. Update has type
`function(Model, Msg): Mvu.Transition<Model, Msg>` and returns the next model
plus commands. View has type
`function(Model, Layout.Viewport): Elements.Element<Msg>` and builds widgets
from that model using `Fpas.Ui.Elements`.

Control IDs stay stable and unique; item keys are local to their collection.
Text, checkbox values, selection and tabs belong to the model. Editor messages
carry text and a directional UTF-16 caret/anchor together. A separate navigation
message changes only selection. Use LF in multiline text and valid grapheme
boundaries. Rejecting a proposal leaves the model value authoritative.

Start the owner loop with
`Gui.Run<Model, Msg>(new Model(), Update, View, new Gui.Options())`.
Options select title, logical width/height, renderer and mailbox capacity.
The host validates the whole active view before drawing. Explicit GUI sizing
uses portable `Layout.Measure.Logical`; the terminal adapter translates those
distances to cells. Layout follows the [shared CUI rules](../../../src/Fpas.Ui/README.md#layout-and-style).
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
selection, focus, theme and owned editor/image counts. `Session.CapturePng(Path)`
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
and grapheme-safe model selections do not imply full bidirectional shaping or
all desktop input methods. One UI thread can own one OpenGL CUI window; Auto
uses software for additional windows on that thread.

## See also

- [Shared widgets and capability profiles](../../../src/Fpas.Ui/README.md)
- [GUI host lifecycle and inspection](../../../src/Fpas.Ui.Gui/README.md)
- [GUI counter](../../../examples/ui/gui/counter/README.md) and [gallery](../../../examples/ui/gui/gallery/README.md)
- [Native assets and local distribution feed](../../build/distribution.md#native-cui-assets)
- [Terminal widget gallery](../../../examples/ui/tui/gallery/README.md)
