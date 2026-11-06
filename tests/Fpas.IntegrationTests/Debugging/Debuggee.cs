using Fpas.Tests.Support;
using System.Text.Json.Nodes;

namespace Fpas.IntegrationTests.Debugging;

internal sealed class Debuggee : IDisposable
{
    internal string DirectoryPath { get; }
    internal string Source => FilePath("Program.fpas");
    internal string Assembly => FilePath("Program.dll");
    private readonly bool preserve;

    internal Debuggee(string name)
    {
        var reports = Environment.GetEnvironmentVariable("FPAS_DEBUGGER_TEST_OUTPUT");
        preserve = !string.IsNullOrWhiteSpace(reports);
        DirectoryPath = Path.Combine(preserve ? Path.GetFullPath(reports!) : Path.GetTempPath(),
            "fpas-dap-" + name + "-" + Guid.NewGuid().ToString("N"));
        Directory.CreateDirectory(DirectoryPath);
    }

    internal string FilePath(string name) => Path.Combine(DirectoryPath, name);

    internal async Task Compile(string source, params string[] options)
    {
        File.WriteAllText(Source, source);
        await CompileFile(Source, Assembly, options);
        File.Copy(Path.Combine(Path.GetDirectoryName(TestEnvironment.Metadata("FpasCliPath"))!, "Fpas.Core.dll"), FilePath("Fpas.Core.dll"));
    }

    internal async Task CompileFile(string source, string assembly, params string[] options)
    {
        var result = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "compile", source,
            "--output", assembly, "--reference-dir", TestEnvironment.ReferenceDirectory, .. options]);
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        Assert.True(File.Exists(Path.ChangeExtension(assembly, ".pdb")));
    }

    internal async Task<string> Run()
    {
        var result = await ProcessRunner.RunAsync([Assembly], workingDirectory: DirectoryPath);
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        return result.Output.Replace("\r\n", "\n", StringComparison.Ordinal);
    }

    internal void UseFullyOptimizedJit()
    {
        var runtimeFile = Path.ChangeExtension(Assembly, ".runtimeconfig.json");
        var runtime = JsonNode.Parse(File.ReadAllText(runtimeFile))!;
        runtime["runtimeOptions"]!["configProperties"] ??= new JsonObject();
        runtime["runtimeOptions"]!["configProperties"]!["System.Runtime.TieredCompilation"] = false;
        File.WriteAllText(runtimeFile, runtime.ToJsonString());
    }

    public void Dispose()
    {
        if (!preserve) TemporaryDirectory.Remove(DirectoryPath);
    }
}
