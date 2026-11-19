// [Documentation](../../../../docs/pascal/libraries/processes.md)
namespace Fpas.IntegrationTests.Processes.Support;

internal sealed class WindowsFactAttribute : FactAttribute
{
    public WindowsFactAttribute()
    {
        if (!OperatingSystem.IsWindows()) Skip = "Native Windows console or filesystem acceptance.";
    }
}
