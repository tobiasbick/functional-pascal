// [Documentation](../../../docs/architecture/workspaces.md#source-edits)
using Fpas.Workspaces.Loading;

namespace Fpas.Workspaces.Editing;

/// <summary>Supplies exact bytes for one checked filesystem change.</summary>
/// <param name="Path">Source file path.</param>
/// <param name="Original">Expected current bytes, or null to require an unoccupied path.</param>
/// <param name="Replacement">New bytes; retain unchanged during publication.</param>
public sealed record SourceChange(string Path, byte[]? Original, byte[] Replacement)
{
    /// <summary>Gets whether replacement bytes differ from the expected original.</summary>
    public bool Changed => Original is null || !Original.AsSpan().SequenceEqual(Replacement);
}

/// <summary>Reports incomplete rollback or cleanup with explicit recovery locations.</summary>
/// <param name="message">Failure explanation.</param>
/// <param name="written">Files still containing published replacements.</param>
/// <param name="backups">Retained recovery or staging paths to inspect.</param>
public sealed class SourcePublicationFailure(string message, IReadOnlyList<string> written, IReadOnlyList<string> backups) : IOException(message)
{
    /// <summary>Gets files whose replacements remain published.</summary>
    public IReadOnlyList<string> Written { get; } = written;
    /// <summary>Gets retained recovery and staging paths.</summary>
    public IReadOnlyList<string> Backups { get; } = backups;
}

/// <summary>Stages source replacements, checks original bytes and rolls back a failed batch.</summary>
public static class SourcePublication
{
    /// <summary>Rejects symbolic links or reparse points anywhere in an existing path.</summary>
    /// <param name="path">Existing source or directory path to validate.</param>
    public static void ValidateRegularPath(string path)
    {
        for (var current = Path.GetFullPath(path); current is not null; current = Path.GetDirectoryName(current))
            if ((File.GetAttributes(current) & FileAttributes.ReparsePoint) != 0)
                throw new ContextFailure("invalid_argument", "Symbolic links and reparse points cannot be edited: " + current);
    }

    /// <summary>Checks regular paths and exact original bytes without writing.</summary>
    /// <param name="changes">Expected source states.</param>
    /// <param name="token">Cancels file reads.</param>
    /// <returns>A task completing when every original still matches.</returns>
    public static async Task VerifyAsync(IEnumerable<SourceChange> changes, CancellationToken token = default)
    {
        foreach (var change in changes)
        {
            token.ThrowIfCancellationRequested();
            if (change.Original is null) { VerifyUnoccupied(change.Path); continue; }
            ValidateRegularPath(change.Path);
            var current = await File.ReadAllBytesAsync(change.Path, token);
            if (!current.AsSpan().SequenceEqual(change.Original))
                throw new ContextFailure("inputs_changed", "Source changed before publication: " + change.Path, inputPath: change.Path);
        }
    }

    internal static void VerifyUnoccupied(string path)
    {
        ValidateRegularPath(Path.GetDirectoryName(path)!);
        try { _ = File.GetAttributes(path); }
        catch (FileNotFoundException) { return; }
        throw new ContextFailure("inputs_changed", "The new source path is already occupied: " + path, inputPath: path);
    }

    /// <summary>Stages a batch, reverifies it and replaces files with rollback on failure.</summary>
    /// <param name="changes">Distinct source paths with authoritative original and replacement bytes.</param>
    /// <param name="verifyContext">Optional final project/input check after staging and before replacement.</param>
    /// <param name="token">Cancels staging and publication; rollback proceeds without cancellation.</param>
    /// <param name="verifyPublished">Optional verification inside the transaction after writing all files.</param>
    /// <returns>A task completing after publication and staging cleanup.</returns>
    public static async Task ApplyAsync(IReadOnlyList<SourceChange> changes,
        Func<CancellationToken, Task>? verifyContext = null, CancellationToken token = default, Func<CancellationToken, Task>? verifyPublished = null)
    {
        var paths = OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal;
        if (changes.Select(change => Path.GetFullPath(change.Path)).Distinct(paths).Count() != changes.Count)
            throw new ContextFailure("edit_conflict", "A publication batch contains duplicate source paths.");
        var staged = new List<(SourceChange Change, string Temporary, string? Backup)>();
        var written = new List<SourceChange>(); var retained = new HashSet<string>(paths);
        try
        {
            await VerifyAsync(changes, token);
            foreach (var change in changes.Where(change => change.Changed))
            {
                token.ThrowIfCancellationRequested();
                if (change.Original is not null && (File.GetAttributes(change.Path) & FileAttributes.ReadOnly) != 0)
                    throw new IOException("Source file is read-only: " + change.Path);
                var stem = Path.Combine(Path.GetDirectoryName(change.Path)!, ".fpas-edit-" + Guid.NewGuid().ToString("N"));
                var backup = change.Original is null ? null : stem + ".bak";
                staged.Add((change, stem + ".new", backup));
                await Stage(stem + ".new", change.Replacement, backup is null ? null : change.Path, token);
                if (backup is not null) await Stage(backup, change.Original!, change.Path, token);
            }
            if (verifyContext is not null) await verifyContext(token);
            await VerifyAsync(changes, token);
            foreach (var item in staged)
            {
                await VerifyAsync([item.Change], token);
                if (item.Backup is null) File.Move(item.Temporary, item.Change.Path, overwrite: false);
                else Replace(item.Temporary, item.Change.Path);
                written.Add(item.Change);
            }
            if (verifyPublished is not null) await verifyPublished(token);
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
                    if (backup is null) File.Delete(change.Path);
                    else Replace(backup, change.Path);
                    written.Remove(change);
                }
                catch (Exception rollback) when (rollback is IOException or UnauthorizedAccessException or ContextFailure)
                { remaining.Add(change.Path); if (backup is not null) retained.Add(backup); }
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
                    if (path is not null && !retained.Contains(path))
                    {
                        try { File.Delete(path); }
                        catch (Exception cleanup) when (cleanup is IOException or UnauthorizedAccessException) { cleanupFailures.Add(path); }
                    }
            if (cleanupFailures.Count != 0)
                throw new SourcePublicationFailure("Could not remove source staging files; inspect the reported recovery paths.",
                    written.Select(change => change.Path).ToArray(), retained.Concat(cleanupFailures).Order(StringComparer.Ordinal).ToArray());
        }
    }

    private static async Task Stage(string path, byte[] bytes, string? source, CancellationToken token)
    {
        await using (var stream = new FileStream(path, FileMode.CreateNew, FileAccess.Write, FileShare.None))
            await stream.WriteAsync(bytes, token);
        if (source is not null && !OperatingSystem.IsWindows()) File.SetUnixFileMode(path, File.GetUnixFileMode(source));
    }

    private static void Replace(string replacement, string source)
    {
        // ReplaceFile preserves the destination's Windows access rules and file identity.
        if (OperatingSystem.IsWindows()) File.Replace(replacement, source, destinationBackupFileName: null);
        else File.Move(replacement, source, overwrite: true);
    }
}
