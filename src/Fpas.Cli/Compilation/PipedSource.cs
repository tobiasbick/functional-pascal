// [Documentation](../../../docs/pascal/program-structure/cli.md#check-one-source-file)
using Fpas.Cli.Analysis;
using Fpas.Compiler;
using Fpas.Workspaces.Loading;
using System.Text;

namespace Fpas.Cli.Compilation;

internal static class PipedSource
{
    internal const string Path = "<stdin>";
    internal const string AssemblyName = "StdinProgram";
    internal const int MaximumBytes = 4 * 1024 * 1024;
    private static readonly UTF8Encoding Utf8 = new(false, true);

    internal static async Task<CompilationSource> ReadAsync(CancellationToken token)
    {
        if (!Console.IsInputRedirected)
            throw new ContextFailure("invalid_argument", "Pipe or redirect UTF-8 FPAS source when using '-' as input.", inputPath: Path);
        var bytes = await BoundedInput.ReadStandardInputAsync(MaximumBytes, "FPAS source on standard input exceeds 4 MiB.", token);
        try { return new(Path, Utf8.GetString(bytes.Span)); }
        catch (DecoderFallbackException)
        { throw new ContextFailure("invalid_argument", "Expected UTF-8 FPAS source on standard input.", inputPath: Path); }
    }
}
