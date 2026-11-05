using Fpas.Workspaces.Loading;

namespace Fpas.Workspaces.Editing;

public sealed record SourceChange(string Path, byte[] Original, byte[] Replacement)
{
    public bool Changed => !Original.AsSpan().SequenceEqual(Replacement);
}

public sealed class SourcePublicationFailure(string message, IReadOnlyList<string> written, IReadOnlyList<string> backups) : IOException(message)
{
    public IReadOnlyList<string> Written { get; } = written;
    public IReadOnlyList<string> Backups { get; } = backups;
}

/// <summary>Stages source replacements, checks original bytes and rolls back a failed batch.</summary>
public static class SourcePublication
{
    public static void ValidateRegularPath(string path)
    {
        for (var current = Path.GetFullPath(path); current is not null; current = Path.GetDirectoryName(current))
            if ((File.GetAttributes(current) & FileAttributes.ReparsePoint) != 0)
                throw new ContextFailure("invalid_argument", "Symbolic links and reparse points cannot be edited: " + current);
    }

    public static async Task VerifyAsync(IEnumerable<SourceChange> changes, CancellationToken token = default)
    {
        foreach (var change in changes)
        {
            token.ThrowIfCancellationRequested(); ValidateRegularPath(change.Path);
            var current = await File.ReadAllBytesAsync(change.Path, token);
            if (!current.AsSpan().SequenceEqual(change.Original))
                throw new ContextFailure("inputs_changed", "Source changed before publication: " + change.Path);
        }
    }

    public static async Task ApplyAsync(IReadOnlyList<SourceChange> changes,
        Func<CancellationToken, Task>? verifyContext = null, CancellationToken token = default)
    {
        var paths = OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;
        if (changes.Select(change => Path.GetFullPath(change.Path)).Distinct(paths).Count() != changes.Count)
            throw new ContextFailure("edit_conflict", "A publication batch contains duplicate source paths.");
        var staged = new List<(SourceChange Change, string Temporary, string Backup)>();
        var written = new List<SourceChange>(); var retained = new HashSet<string>(paths);
        try
        {
            await VerifyAsync(changes, token);
            foreach (var change in changes.Where(change => change.Changed))
            {
                token.ThrowIfCancellationRequested();
                if ((File.GetAttributes(change.Path) & FileAttributes.ReadOnly) != 0)
                    throw new IOException("Source file is read-only: " + change.Path);
                var stem = Path.Combine(Path.GetDirectoryName(change.Path)!, ".fpas-edit-" + Guid.NewGuid().ToString("N"));
                staged.Add((change, stem + ".new", stem + ".bak"));
                await Stage(stem + ".new", change.Replacement, change.Path, token);
                await Stage(stem + ".bak", change.Original, change.Path, token);
            }
            if (verifyContext is not null) await verifyContext(token);
            await VerifyAsync(changes, token);
            foreach (var item in staged)
            {
                await VerifyAsync([item.Change], token);
                Replace(item.Temporary, item.Change.Path);
                written.Add(item.Change);
            }
        }
        catch (Exception failure)
        {
            var remaining = new List<string>();
            foreach (var change in written.AsEnumerable().Reverse().ToArray())
            {
                var backup = staged.Single(item => ReferenceEquals(item.Change, change)).Backup;
                try
                {
                    // Do not overwrite another writer's changes during rollback.
                    await VerifyAsync([change with { Original = change.Replacement }], CancellationToken.None);
                    Replace(backup, change.Path);
                    written.Remove(change);
                }
                catch (Exception rollback) when (rollback is IOException or UnauthorizedAccessException or ContextFailure)
                { remaining.Add(change.Path); retained.Add(backup); }
            }
            if (remaining.Count != 0)
                throw new SourcePublicationFailure("Source publication failed and rollback could not restore every file. " + failure.Message,
                    remaining.AsReadOnly(), retained.Order(StringComparer.Ordinal).ToArray());
            throw;
        }
        finally
        {
            var cleanupFailures = new List<string>();
            foreach (var item in staged)
                foreach (var path in new[] { item.Temporary, item.Backup })
                    if (!retained.Contains(path))
                    {
                        try { File.Delete(path); }
                        catch (Exception cleanup) when (cleanup is IOException or UnauthorizedAccessException) { cleanupFailures.Add(path); }
                    }
            if (cleanupFailures.Count != 0)
                throw new SourcePublicationFailure("Could not remove source staging files; inspect the reported recovery paths.",
                    written.Select(change => change.Path).ToArray(), retained.Concat(cleanupFailures).Order(StringComparer.Ordinal).ToArray());
        }
    }

    private static async Task Stage(string path, byte[] bytes, string source, CancellationToken token)
    {
        await using (var stream = new FileStream(path, FileMode.CreateNew, FileAccess.Write, FileShare.None))
            await stream.WriteAsync(bytes, token);
        if (!OperatingSystem.IsWindows()) File.SetUnixFileMode(path, File.GetUnixFileMode(source));
    }

    private static void Replace(string replacement, string source)
    {
        // ReplaceFile preserves the destination's Windows access rules and file identity.
        if (OperatingSystem.IsWindows()) File.Replace(replacement, source, destinationBackupFileName: null);
        else File.Move(replacement, source, overwrite: true);
    }
}
