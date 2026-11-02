using Fpas.Tests.Support;

namespace Fpas.IntegrationTests;

public sealed class DebugSymbolsCliTests
{
    [Fact]
    public async Task CLI_publishes_source_stack_traces_and_removes_stale_symbols_when_disabled()
    {
        using var directory = new TemporaryDirectory();
        var source = directory.File("Trace.fpas"); var output = directory.File("Trace.dll"); var pdb = Path.ChangeExtension(output, ".pdb");
        File.WriteAllText(source, """
            async program Trace;
            uses System as Sys, System.Threading.Tasks as Tasks;
            async function Crash(): Tasks.Task;
            begin
              await Tasks.Task.Delay(1);
              raise new Sys.ArgumentException('trace');
            end function;
            begin await Crash(); end;
            end program;
            """);
        async Task<ProcessResult> Compile(params string[] options) => await ProcessRunner.RunAsync([
            TestEnvironment.Metadata("FpasCliPath"), "compile", source, "--output", output,
            "--reference-dir", TestEnvironment.ReferenceDirectory, .. options]);
        var built = await Compile(); Assert.True(built.ExitCode == 0, built.Output + built.Error); Assert.True(File.Exists(pdb));
        var assembly = File.ReadAllBytes(output); var symbols = File.ReadAllBytes(pdb);
        var failed = await ProcessRunner.RunAsync([output]);
        Assert.NotEqual(0, failed.ExitCode); Assert.True(failed.Error.Contains(source + ":line 6", StringComparison.Ordinal), failed.Error);
        File.AppendAllText(source, "invalid");
        Assert.NotEqual(0, (await Compile()).ExitCode);
        Assert.Equal(assembly, File.ReadAllBytes(output)); Assert.Equal(symbols, File.ReadAllBytes(pdb));
        File.WriteAllText(source, "program Trace; begin end; end program;");
        var disabled = await Compile("--debug", "none"); Assert.True(disabled.ExitCode == 0, disabled.Output + disabled.Error);
        Assert.False(File.Exists(pdb));
        Directory.CreateDirectory(pdb);
        Assert.NotEqual(0, (await Compile()).ExitCode);
        Assert.True(Directory.Exists(pdb)); Assert.Empty(Directory.GetDirectories(directory.Path, ".fpas-*"));
    }
}
