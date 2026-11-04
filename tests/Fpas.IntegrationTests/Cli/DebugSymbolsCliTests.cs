using Fpas.Tests.Support;
using System.Reflection.Metadata;
using System.Diagnostics;

namespace Fpas.IntegrationTests.Cli;

public sealed class DebugSymbolsCliTests
{
    [Fact]
    public async Task Optimization_retains_symbols_without_disabling_the_JIT()
    {
        using var directory = new TemporaryDirectory();
        var source = directory.File("Main.fpas"); var output = directory.File("Main.dll");
        File.WriteAllText(source, "program Main; begin end; end program;");
        async Task Compile(params string[] options)
        {
            var result = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "compile", source,
                "--output", output, "--reference-dir", TestEnvironment.ReferenceDirectory, .. options]);
            Assert.True(result.ExitCode == 0, result.Output + result.Error);
        }
        await Compile();
        Assert.True(AssemblyInspection.DebuggingModes(output).HasFlag(DebuggableAttribute.DebuggingModes.DisableOptimizations));
        await Compile("--optimize");
        Assert.False(AssemblyInspection.DebuggingModes(output).HasFlag(DebuggableAttribute.DebuggingModes.DisableOptimizations));
        Assert.True(File.Exists(Path.ChangeExtension(output, ".pdb")));
        Assert.Equal(0, (await ProcessRunner.RunAsync([output])).ExitCode);
    }

    [Fact]
    public async Task Relative_CLI_sources_record_absolute_documents_for_other_runtime_working_directories()
    {
        using var directory = new TemporaryDirectory();
        var source = directory.File("Trace.fpas");
        var output = directory.File("output/Trace.dll");
        File.WriteAllText(source, """
            program Trace;
            uses System as Sys;
            begin
              raise new Sys.ArgumentException('trace');
            end;
            end program;
            """);
        var built = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "compile", "Trace.fpas",
            "--output", "output/Trace.dll", "--reference-dir", TestEnvironment.ReferenceDirectory], workingDirectory: directory.Path);
        Assert.True(built.ExitCode == 0, built.Output + built.Error);
        using var stream = File.OpenRead(Path.ChangeExtension(output, ".pdb"));
        using var provider = MetadataReaderProvider.FromPortablePdbStream(stream);
        var symbols = provider.GetMetadataReader();
        Assert.Equal(source, symbols.GetString(symbols.GetDocument(Assert.Single(symbols.Documents)).Name));
        var failed = await ProcessRunner.RunAsync([output], workingDirectory: Path.GetDirectoryName(output));
        Assert.NotEqual(0, failed.ExitCode);
        Assert.Contains(source + ":line 4", failed.Error, StringComparison.Ordinal);
    }

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
