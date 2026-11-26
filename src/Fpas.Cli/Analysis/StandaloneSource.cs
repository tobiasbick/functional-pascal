// [Documentation](../../../docs/pascal/program-structure/cli.md#check-one-source-file)
using Fpas.Compiler;
using Fpas.Compiler.Analysis;
using Fpas.Cli.Compilation;
using Fpas.Cli.Toolchain;
using Fpas.Workspaces.Loading;
using System.Text;

namespace Fpas.Cli.Analysis;

internal sealed class StandaloneSource(CompilationSource source, string assembly, byte[]? input)
{
    private static readonly UTF8Encoding Utf8 = new(false, true);
    internal CompilationSource Source => source;

    internal static async Task<StandaloneSource> OpenAsync(string path, CancellationToken token)
    {
        var input = path == PipedSource.Path ? null : await BoundedFile.ReadAsync(path, PipedSource.MaximumBytes, token);
        var source = input is null ? await PipedSource.ReadAsync(token) : new CompilationSource(path, Utf8.GetString(input));
        var assembly = input is null ? PipedSource.AssemblyName : Path.GetFileNameWithoutExtension(path);
        return new(source, assembly, input);
    }

    internal AnalysisRequest Request(CompilationTarget target = CompilationTarget.Exe)
        => new([source], assembly, FrameworkReferences.Resolve(), target);

    internal async Task VerifyAsync(CancellationToken token)
    {
        if (input is null) return;
        var current = await BoundedFile.ReadAsync(source.Path, PipedSource.MaximumBytes, token);
        if (!input.AsSpan().SequenceEqual(current))
            throw new ContextFailure("inputs_changed", "The source changed during analysis. Retry with the current source.", inputPath: source.Path);
    }
}
