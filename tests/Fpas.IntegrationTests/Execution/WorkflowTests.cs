using Fpas.IntegrationTests.Debugging;

namespace Fpas.IntegrationTests.Execution;

/// <summary>Exercises deferred workflows through the FPAS source interface.</summary>
public sealed class WorkflowTests
{
    /// <summary>Compiles the library with one program and runs its contracts.</summary>
    /// <param name="scenario">The program exercising the workflow interface.</param>
    /// <returns>The result of compilation and execution.</returns>
    [Theory]
    [InlineData("Operations")]
    [InlineData("Lifetimes")]
    [InlineData("Streams")]
    public async Task Workflows_follow_their_public_contracts(string scenario)
    {
        var fixtures = Path.Combine(AppContext.BaseDirectory, "Fixtures", "workflows");
        var sources = Directory.GetFiles(Path.Combine(fixtures, "lib"), "*.fpas");
        using var program = new Debuggee("workflows-" + scenario);
        await program.Compile(await File.ReadAllTextAsync(Path.Combine(fixtures, scenario + ".fpas")), sources);
        Assert.Equal("Workflow contracts verified\n", await program.Run());
    }
}
