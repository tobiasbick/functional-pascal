// [Documentation](../../../docs/pascal/libraries/processes.md#reading-files-for-tool-input)
using Fpas.Processes;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Processes;

/// <summary>Exercises the regular-file input shared by the CLI and debugger.</summary>
public sealed class RegularFileTests
{
    /// <summary>Checks ordinary reads and a symbolic link to the same file on Linux.</summary>
    [Fact]
    public async Task Reads_regular_file_bytes()
    {
        using var directory = new TemporaryDirectory();
        var path = directory.File("input");
        byte[] bytes = [0, 1, 255, 42];
        File.WriteAllBytes(path, bytes);
        if (OperatingSystem.IsLinux())
        {
            var link = directory.File("input-link");
            File.CreateSymbolicLink(link, path);
            path = link;
        }

        await using var stream = RegularFile.OpenRead(path);
        Assert.Equal(bytes.Length, stream.Length);
        var read = new byte[bytes.Length];
        await stream.ReadExactlyAsync(read);
        Assert.Equal(bytes, read);
    }

    /// <summary>Checks that a directory cannot provide bytes as a file input.</summary>
    [Fact]
    public void Rejects_a_directory()
    {
        using var directory = new TemporaryDirectory();
        var failure = Record.Exception(() =>
        {
            using var stream = RegularFile.OpenRead(directory.Path);
        });
        Assert.True(failure is IOException or UnauthorizedAccessException, failure?.ToString());
    }

    /// <summary>Checks that the opener respects the platform's normal file sharing rules.</summary>
    [Fact]
    public void Respects_exclusive_file_sharing()
    {
        using var directory = new TemporaryDirectory();
        var path = directory.File("input");
        using var writer = File.Open(path, FileMode.CreateNew, FileAccess.Write, FileShare.None);
        var ordinary = Record.Exception(() => File.OpenRead(path).Dispose());
        var checkedOpen = Record.Exception(() => RegularFile.OpenRead(path).Dispose());
        Assert.Equal(ordinary?.GetType(), checkedOpen?.GetType());
    }
}
