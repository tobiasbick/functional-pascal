using Fpas.Tests.Support;

namespace Fpas.IntegrationTests;

internal static class ProgramExecution
{
    internal static async Task<ProcessResult> Run(string source, params string[] arguments)
    {
        using var directory = new TemporaryDirectory();
        var output = directory.File("Program.dll");
        var compiled = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "compile", "-", "--output", output,
            "--reference-dir", TestEnvironment.ReferenceDirectory, "--diagnostics", "json"], source);
        Assert.True(compiled.ExitCode == 0, compiled.Output + compiled.Error);
        return await ProcessRunner.RunAsync([output, .. arguments], environment: new Dictionary<string, string?>
        {
            ["DOTNET_SYSTEM_GLOBALIZATION_INVARIANT"] = "1"
        });
    }

    internal static async Task Output(string source, params string[] lines)
    {
        var result = await Run(source);
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        Assert.Equal(string.Join(Environment.NewLine, lines) + Environment.NewLine, result.Output);
        Assert.Equal("", result.Error);
    }
}
