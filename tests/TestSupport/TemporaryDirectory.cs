namespace Fpas.Tests.Support;

internal sealed class TemporaryDirectory : IDisposable
{
    internal string Path { get; } = System.IO.Path.Combine(System.IO.Path.GetTempPath(), "fpas tests " + Guid.NewGuid().ToString("N"));

    internal TemporaryDirectory() => Directory.CreateDirectory(Path);
    internal string File(string name) => System.IO.Path.Combine(Path, name);
    public void Dispose() => Directory.Delete(Path, recursive: true);
}
