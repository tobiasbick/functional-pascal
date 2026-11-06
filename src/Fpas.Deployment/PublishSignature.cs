using System.Security.Cryptography;
using System.Security.Cryptography.Pkcs;
using System.Security.Cryptography.X509Certificates;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Fpas.Deployment;

internal sealed record SignedFile(string Path, long Bytes, string Sha256);
internal sealed record SignedFiles(int Version, SignedFile[] Files);

public static class PublishSignature
{
    internal const string Name = ".fpas-signature.p7s";

    internal static SignedFile[] Inventory(string directory)
    {
        var files = new List<SignedFile>(); var folders = new Queue<string>(); folders.Enqueue(directory);
        while (folders.TryDequeue(out var folder))
        {
            foreach (var path in Directory.EnumerateFileSystemEntries(folder))
            {
                var attributes = File.GetAttributes(path);
                if ((attributes & FileAttributes.ReparsePoint) != 0) throw new IOException("Signed publish output cannot contain symbolic links.");
                if ((attributes & FileAttributes.Directory) != 0) folders.Enqueue(path);
                else
                {
                    var relative = Path.GetRelativePath(directory, path).Replace('\\', '/');
                    if (relative is Name or ".fpas-publish.json") continue;
                    if (files.Count >= 20_000) throw new IOException("Signed output exceeds 20000 files.");
                    using var input = File.OpenRead(path);
                    files.Add(new(relative, input.Length, Convert.ToHexString(SHA256.HashData(input))));
                }
            }
        }
        return files.OrderBy(file => file.Path, StringComparer.Ordinal).ToArray();
    }

    public static void Sign(string directory, string certificate, string passwordEnvironment)
    {
        var password = passwordEnvironment.Length == 0 ? null : Environment.GetEnvironmentVariable(passwordEnvironment)
            ?? throw new CryptographicException("The certificate password environment variable is absent.");
        using var identity = X509CertificateLoader.LoadPkcs12FromFile(certificate, password, X509KeyStorageFlags.EphemeralKeySet);
        if (!identity.HasPrivateKey) throw new CryptographicException("The publishing certificate needs a private key.");
        var files = Inventory(directory);
        if (files.Length == 0) throw new IOException("There are no published files to sign.");
        var manifest = JsonSerializer.SerializeToUtf8Bytes(new(1, files), SignedFilesJson.Default.SignedFiles);
        var message = new SignedCms(new ContentInfo(manifest));
        message.ComputeSignature(new CmsSigner(SubjectIdentifierType.IssuerAndSerialNumber, identity)
        { DigestAlgorithm = new Oid("2.16.840.1.101.3.4.2.1"), IncludeOption = X509IncludeOption.EndCertOnly }, silent: true);
        var encoded = message.Encode();
        if (encoded.Length > 8 * 1024 * 1024) throw new IOException("Publish signature exceeds 8 MiB.");
        var target = Path.Combine(directory, Name); var temporary = target + "." + Guid.NewGuid().ToString("N") + ".tmp";
        try { File.WriteAllBytes(temporary, encoded); File.Move(temporary, target, overwrite: true); }
        finally { File.Delete(temporary); }
    }

    public static void Verify(string directory, string certificate)
    {
        var file = Path.Combine(directory, Name);
        if (new FileInfo(file).Length > 8 * 1024 * 1024) throw new IOException("Publish signature exceeds 8 MiB.");
        using var expected = X509CertificateLoader.LoadCertificateFromFile(certificate);
        var message = new SignedCms(); message.Decode(File.ReadAllBytes(file)); message.CheckSignature(verifySignatureOnly: true);
        if (message.SignerInfos.Count != 1 || message.SignerInfos[0].Certificate is not { } signer || !signer.RawData.AsSpan().SequenceEqual(expected.RawData))
            throw new CryptographicException("Publish signature does not match the expected certificate.");
        var manifest = JsonSerializer.Deserialize(message.ContentInfo.Content, SignedFilesJson.Default.SignedFiles);
        if (manifest is not { Version: 1, Files: not null } || !manifest.Files.SequenceEqual(Inventory(directory)))
            throw new CryptographicException("Published files differ from the signed inventory.");
    }
}

[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase)]
[JsonSerializable(typeof(SignedFiles))]
internal partial class SignedFilesJson : JsonSerializerContext;
