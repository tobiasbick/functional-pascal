using Fpas.Debugging.Batch;

namespace Fpas.Debugging.Failures;

internal sealed class FailureReportFile : IDisposable
{
    private readonly string destination;
    private readonly string pending;
    internal bool Saved { get; private set; }

    internal FailureReportFile(string destination)
    {
        if (!Path.IsPathFullyQualified(destination)) throw new ArgumentException("Use an absolute failure report path.");
        if (Path.Exists(destination)) throw new ArgumentException("The failure report destination must not already exist.");
        this.destination = destination;
        Directory.CreateDirectory(Path.GetDirectoryName(destination)!);
        pending = destination + "." + Guid.NewGuid().ToString("N") + ".tmp";
        // Validate writable storage before executing any target statements.
        using var prepared = new FileStream(pending, FileMode.CreateNew, FileAccess.Write, FileShare.None);
    }

    internal async Task Save(BatchReport report)
    {
        await File.WriteAllBytesAsync(pending, report.Serialize(), CancellationToken.None);
        File.Move(pending, destination, overwrite: Saved);
        Saved = true;
    }

    /// <summary>Remove the unused staging file while preserving any published report.</summary>
    public void Dispose() => File.Delete(pending);
}
