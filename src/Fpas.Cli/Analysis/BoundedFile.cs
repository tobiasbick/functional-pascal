// [Documentation](../../../docs/pascal/program-structure/cli.md)
using Fpas.Processes;
using Fpas.Workspaces.Loading;

namespace Fpas.Cli.Analysis;

internal static class BoundedFile
{
    internal static async Task<byte[]> ReadAsync(string path, int maximum, CancellationToken token)
    {
        token.ThrowIfCancellationRequested();
        await using var input = RegularFile.OpenRead(path);
        var length = input.Length;
        if (length > maximum)
            throw new ContextFailure("invalid_argument", $"Input exceeds {maximum / (1024 * 1024)} MiB.", inputPath: path);
        var bytes = new byte[(int)length];
        try { await input.ReadExactlyAsync(bytes, token); }
        catch (EndOfStreamException) { throw Changed(path); }
        if (await input.ReadAsync(new byte[1], token) != 0) throw Changed(path);
        return bytes;
    }

    private static ContextFailure Changed(string path)
        => new("inputs_changed", "The input length changed during capture.", inputPath: path);
}
