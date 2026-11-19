// [Documentation](../../coverage.md#behavioral-checks)
namespace Fpas.IntegrationTests.Support;

// Reports a fact as skipped on Windows, where creating symbolic links can require elevated privileges.
internal sealed class UnixFactAttribute : FactAttribute
{
    public UnixFactAttribute()
    {
        if (OperatingSystem.IsWindows()) Skip = UnixTest.SkipReason;
    }
}

// Reports a theory as skipped on Windows, where creating symbolic links can require elevated privileges.
internal sealed class UnixTheoryAttribute : TheoryAttribute
{
    public UnixTheoryAttribute()
    {
        if (OperatingSystem.IsWindows()) Skip = UnixTest.SkipReason;
    }
}

internal static class UnixTest
{
    internal const string SkipReason = "Symbolic link acceptance; creating links on Windows can require developer mode or elevation.";
}
