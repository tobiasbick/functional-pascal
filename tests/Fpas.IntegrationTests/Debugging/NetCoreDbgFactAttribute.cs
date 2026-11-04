namespace Fpas.IntegrationTests.Debugging;

internal sealed class NetCoreDbgFactAttribute : FactAttribute
{
    public NetCoreDbgFactAttribute()
    {
        if (string.IsNullOrWhiteSpace(Environment.GetEnvironmentVariable("FPAS_NETCOREDBG_PATH")))
            Skip = "Set FPAS_NETCOREDBG_PATH to the tested NetCoreDbg executable; see tests/README.md.";
    }
}
