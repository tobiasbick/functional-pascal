using Fpas.Workspaces.Editing;
using System.Text;

namespace Fpas.Cli.Formatting;

internal sealed record FormatSnapshot(string Path, byte[] Original, string Formatted)
{
    internal SourceChange Change => new(Path, Original, FormatPublication.Utf8.GetBytes(Formatted));
    internal bool Changed => Change.Changed;
}

internal static class FormatPublication
{
    internal static readonly UTF8Encoding Utf8 = new(false, true);

    internal static async Task Verify(IEnumerable<FormatSnapshot> snapshots, CancellationToken token)
        => await SourcePublication.VerifyAsync(snapshots.Select(snapshot => snapshot.Change), token);

    internal static async Task Write(FormatSnapshot snapshot, CancellationToken token)
        => await SourcePublication.ApplyAsync([snapshot.Change], token: token);
}
