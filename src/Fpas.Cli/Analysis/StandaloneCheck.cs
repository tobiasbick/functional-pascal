// [Documentation](../../../docs/pascal/program-structure/cli.md#check-one-source-file)
using Fpas.Compiler;
using Fpas.Cli.Compilation;
using Fpas.Cli.Toolchain;
using Fpas.Workspaces.Loading;
using System.Text;

namespace Fpas.Cli.Analysis;

internal static class StandaloneCheck
{
    private static readonly UTF8Encoding Utf8 = new(false, true);

    internal static async Task<CheckResult> AnalyzeAsync(string path, string configuration, CancellationToken token)
    {
        var input = path == PipedSource.Path ? null : await BoundedFile.ReadAsync(path, PipedSource.MaximumBytes, token);
        var source = input is null ? await PipedSource.ReadAsync(token) : new CompilationSource(path, Utf8.GetString(input));
        var assembly = input is null ? PipedSource.AssemblyName : Path.GetFileNameWithoutExtension(path);
        var analysis = FpasCompiler.Analyze(new([source], assembly, FrameworkReferences.Resolve()), token);
        if (input is not null)
        {
            var current = await BoundedFile.ReadAsync(path, PipedSource.MaximumBytes, token);
            if (!input.AsSpan().SequenceEqual(current))
                throw new ContextFailure("inputs_changed", "The source changed during analysis. Run fpas check again.", inputPath: path);
        }
        return CheckResult.FromSource(source, configuration, analysis);
    }
}
