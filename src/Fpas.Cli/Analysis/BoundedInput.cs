// [Documentation](../../../docs/pascal/program-structure/project-commands.md#run)
using Fpas.Workspaces.Loading;

namespace Fpas.Cli.Analysis;

internal static class BoundedInput
{
    internal static async Task<ReadOnlyMemory<byte>> ReadStandardInputAsync(int maximum, string limitMessage, CancellationToken token)
    {
        token.ThrowIfCancellationRequested();
        using var stdin = Console.OpenStandardInput();
        var bytes = new byte[maximum + 1];
        var length = 0;
        while (length < bytes.Length)
        {
            var read = await stdin.ReadAsync(bytes.AsMemory(length), token).AsTask().WaitAsync(token);
            if (read == 0) break;
            length += read;
        }
        if (length > maximum) throw new ContextFailure("invalid_argument", limitMessage);
        return bytes.AsMemory(0, length);
    }
}
