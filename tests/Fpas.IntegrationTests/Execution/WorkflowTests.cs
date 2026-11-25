// [Documentation](../../../docs/pascal/libraries/workflows.md)
using Fpas.IntegrationTests.Debugging;
using Fpas.Processes;

namespace Fpas.IntegrationTests.Execution;

/// <summary>Exercises deferred workflows through the FPAS source interface.</summary>
public sealed class WorkflowTests
{
    private static string Fixtures => Path.Combine(AppContext.BaseDirectory, "Fixtures", "workflows");
    private static string[] Library => [Path.Combine(AppContext.BaseDirectory, "Fixtures", "effects", "Effect.fpas"),
        .. Directory.GetFiles(Path.Combine(Fixtures, "lib"), "*.fpas")];

    /// <summary>Compiles the library with one program and runs its contracts.</summary>
    /// <param name="scenario">The program exercising the workflow interface.</param>
    /// <returns>The result of compilation and execution.</returns>
    [Theory]
    [InlineData("Operations")]
    [InlineData("Lifetimes")]
    [InlineData("Streams")]
    public async Task Workflows_follow_their_public_contracts(string scenario)
    {
        using var program = new Debuggee("workflows-" + scenario);
        await program.Compile(await File.ReadAllTextAsync(Path.Combine(Fixtures, scenario + ".fpas")), Library);
        Assert.Equal("Workflow contracts verified\n", await program.Run());
    }

    /// <summary>Checks that both captured streams are decoded using UTF-8.</summary>
    /// <returns>The result of compilation and execution.</returns>
    [Fact]
    public async Task Process_capture_decodes_both_streams_as_utf8()
    {
        var backend = typeof(OwnedProcess).Assembly.Location;
        using var program = new Debuggee("workflows-capture");
        await program.Compile(await File.ReadAllTextAsync(Path.Combine(Fixtures, "CaptureEncoding.fpas")),
            [.. Library, Path.Combine(Fixtures, "process", "Capture.fpas"), "--reference", backend]);
        File.Copy(backend, program.FilePath("Fpas.Processes.dll"));
        Assert.Equal("Capture encoding verified\n", await program.Run());
    }
}
