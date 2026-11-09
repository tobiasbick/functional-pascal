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
