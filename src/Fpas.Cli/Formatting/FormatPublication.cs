using Fpas.Workspaces.Formatting;
using System.Text;

namespace Fpas.Cli.Formatting;

internal sealed record FormatSnapshot(string Path, byte[] Original, string Formatted)
{
    internal bool Changed => !Original.AsSpan().SequenceEqual(FormatPublication.Utf8.GetBytes(Formatted));
}

internal static class FormatPublication
{
    internal static readonly UTF8Encoding Utf8 = new(false, true);

    internal static async Task Verify(IEnumerable<FormatSnapshot> snapshots, CancellationToken token)
    {
        foreach (var snapshot in snapshots)
        {
            FormatSources.ValidateRegularPath(snapshot.Path);
            var current = await File.ReadAllBytesAsync(snapshot.Path, token);
            if (!current.AsSpan().SequenceEqual(snapshot.Original)) throw new IOException("Source changed during formatting: " + snapshot.Path);
        }
    }

    internal static async Task Write(FormatSnapshot snapshot, CancellationToken token)
    {
        FormatSources.ValidateRegularPath(snapshot.Path);
        if ((File.GetAttributes(snapshot.Path) & FileAttributes.ReadOnly) != 0) throw new IOException("Source file is read-only: " + snapshot.Path);
        var temporary = Path.Combine(Path.GetDirectoryName(snapshot.Path)!, ".fpas-fmt-" + Guid.NewGuid().ToString("N"));
        try
        {
            await using (var stream = new FileStream(temporary, FileMode.CreateNew, FileAccess.Write, FileShare.None))
                await stream.WriteAsync(Utf8.GetBytes(snapshot.Formatted), token);
            if (!OperatingSystem.IsWindows()) File.SetUnixFileMode(temporary, File.GetUnixFileMode(snapshot.Path));
            await Verify([snapshot], token);
            token.ThrowIfCancellationRequested();
            File.Move(temporary, snapshot.Path, overwrite: true);
        }
        finally { if (File.Exists(temporary)) File.Delete(temporary); }
    }
}
