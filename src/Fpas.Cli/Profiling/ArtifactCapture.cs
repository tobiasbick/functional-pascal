namespace Fpas.Cli.Profiling;

// Collect into an invocation-owned location; publication never replaces an existing artifact.
internal sealed class ArtifactCapture
{
    internal string Destination { get; }
    internal string StagedPath { get; }
    internal ArtifactCapture(string destination)
    {
        Destination = Path.GetFullPath(destination);
        if (File.Exists(Destination) || Directory.Exists(Destination)) throw new ArgumentException("Output already exists. Choose a fresh artifact filename.");
        var parent = Path.GetDirectoryName(Destination)!; Directory.CreateDirectory(parent);
        var staging = Path.Combine(parent, ".fpas-artifact-" + Guid.NewGuid().ToString("N")); Directory.CreateDirectory(staging);
        StagedPath = Path.Combine(staging, Path.GetFileName(Destination));
    }
    internal string Publish()
    {
        if (!File.Exists(StagedPath) || new FileInfo(StagedPath).Length == 0) throw new IOException("The collector produced no nonempty artifact.");
        File.Move(StagedPath, Destination, overwrite: false);
        Directory.Delete(Path.GetDirectoryName(StagedPath)!);
        return Destination;
    }
    internal string? Partial => File.Exists(StagedPath) ? StagedPath : null;
}
