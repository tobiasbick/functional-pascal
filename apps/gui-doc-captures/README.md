# GUI documentation captures

This app drives the real FPAS gallery and produces widget images for the
[GUI handbook](../../docs/ui/gui/README.md). Run it from the repository root
with the local FPAS SDK. Capturing uses a display, and cropping runs headlessly.

```sh
dotnet run --project apps/gui-doc-captures/GuiDocCaptures.fpasproj -- .temp-data/ui-selection selection
dotnet run --project apps/gui-doc-captures/GuiDocCaptures.fpasproj -- --crop .temp-data/ui-selection docs/ui/gui/selection/images
```

The [capture guide](doc/captures.md) lists supported chapters, output files
and handbook destinations. It also explains how to [run the app tests](doc/captures.md#verification).
