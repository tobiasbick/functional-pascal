namespace Fpas.Tests.Support;

internal sealed class TemporaryDirectory : IDisposable
{
    internal string Path { get; } = System.IO.Path.Combine(System.IO.Path.GetTempPath(), "fpas tests " + Guid.NewGuid().ToString("N"));

    internal TemporaryDirectory() => Directory.CreateDirectory(Path);
    internal string File(string name) => System.IO.Path.GetFullPath(System.IO.Path.Combine(Path, name));
    public void Dispose() => Remove(Path);

    internal static void Remove(string path)
    {
        var deadline = Environment.TickCount64 + 5000;
        while (true)
        {
            try { Directory.Delete(path, recursive: true); return; }
            catch (Exception exception) when (OperatingSystem.IsWindows()
                && (exception is IOException or UnauthorizedAccessException) && Environment.TickCount64 < deadline)
            {
                // Windows can briefly retain image-file locks after a terminated process has exited.
                Thread.Sleep(20);
            }
        }
    }
}
