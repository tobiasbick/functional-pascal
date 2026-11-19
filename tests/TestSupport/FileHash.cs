// [Documentation](../coverage.md#test-harness)
using System.Security.Cryptography;

namespace Fpas.Tests.Support;

internal static class FileHash
{
    /// <summary>Returns the uppercase hexadecimal SHA-256 digest of a file.</summary>
    /// <param name="path">File to hash.</param>
    /// <returns>The digest in the form used by FPAS publish manifests.</returns>
    internal static string Sha256(string path)
    {
        using var stream = File.OpenRead(path);
        return Convert.ToHexString(SHA256.HashData(stream));
    }
}
