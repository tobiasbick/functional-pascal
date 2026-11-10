namespace Fpas.Gui.Cui.Tests;

internal static class GuiAvailability
{
    internal static string? SkipReason => OperatingSystem.IsLinux() &&
        string.IsNullOrWhiteSpace(Environment.GetEnvironmentVariable("DISPLAY"))
        ? "Native window tests require DISPLAY; run them under xvfb-run on headless Linux."
        : null;
}

internal sealed class GuiFactAttribute : FactAttribute
{
    public GuiFactAttribute() => Skip = GuiAvailability.SkipReason;
}

internal sealed class GuiTheoryAttribute : TheoryAttribute
{
    public GuiTheoryAttribute() => Skip = GuiAvailability.SkipReason;
}

internal sealed class X11GuiFactAttribute : FactAttribute
{
    public X11GuiFactAttribute() => Skip = !OperatingSystem.IsLinux()
        ? "This OS event-path check uses X11/XTest on Linux."
        : GuiAvailability.SkipReason;
}

internal sealed class GuiPublishingFactAttribute : FactAttribute
{
    public GuiPublishingFactAttribute() => Skip = GuiAvailability.SkipReason ??
        (Environment.GetEnvironmentVariable("FPAS_TEST_GUI_AOT") == "1"
            ? null
            : "Set FPAS_TEST_GUI_AOT=1 to run the GUI publishing test with the native toolchain.");
}
