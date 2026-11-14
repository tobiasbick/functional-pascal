using System.Security.Cryptography;
using System.Text.Json;
using Fpas.Tests.Support;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Projects;

namespace Fpas.IntegrationTests.Projects;

/// <summary>Checks publish ownership without producing a delivery.</summary>
public sealed class PublishDirectoryTests
{
    /// <summary>Accepts a legacy manifest and refuses an unrecorded empty directory.</summary>
    [Fact]
    public async Task OwnershipRejectsAnAddedEmptyDirectory()
    {
        using var directory = new TemporaryDirectory();
        byte[] output = [1, 2, 3];
        await File.WriteAllBytesAsync(directory.File("app.dll"), output);
        var manifest = directory.File(".fpas-publish.json");
        var files = new[] { new { path = "app.dll", bytes = output.Length,
            sha256 = Convert.ToHexString(SHA256.HashData(output)) } };
        await File.WriteAllTextAsync(manifest, JsonSerializer.Serialize(new { version = 1, files }));
        await PublishDirectory.Check(directory.Path, CancellationToken.None);

        var empty = Directory.CreateDirectory(directory.File("notes/empty"));
        var conflict = await Assert.ThrowsAsync<ContextFailure>(() =>
            PublishDirectory.Check(directory.Path, CancellationToken.None));
        Assert.Equal("output_conflict", conflict.Code);
        Assert.True(empty.Exists);

        await File.WriteAllTextAsync(manifest, JsonSerializer.Serialize(new { version = 1, files,
            emptyDirectories = new[] { "notes/empty" } }));
        await PublishDirectory.Check(directory.Path, CancellationToken.None);
    }
}
