using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using System.Security.Cryptography;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Projects;

public sealed partial class PublishingTests
{
    [Fact]
    public async Task SDK_signing_is_cryptographic_incremental_and_never_deploys_the_private_key()
    {
        using var directory = new TemporaryDirectory(); var project = await Create(directory);
        using var rsa = new RSACryptoServiceProvider(2048);
        var key = directory.File("App/key.snk"); File.WriteAllBytes(key, rsa.ExportCspBlob(true));
        var xml = System.Xml.Linq.XDocument.Load(project);
        xml.Root!.Element("PropertyGroup")!.Add(new System.Xml.Linq.XElement("SignAssembly", "true"), new System.Xml.Linq.XElement("AssemblyOriginatorKeyFile", "key.snk"));
        xml.Root.Add(new System.Xml.Linq.XElement("ItemGroup", new System.Xml.Linq.XElement("None", new System.Xml.Linq.XAttribute("Update", "key.snk"), new System.Xml.Linq.XAttribute("CopyToPublishDirectory", "Always")))); xml.Save(project);
        var output = directory.File("App/bin/Release/net10.0/App.dll");
        async Task<ProcessResult> Build(params string[] extra) => await ProcessRunner.RunAsync(["build", project, "-c", "Release", .. extra], workingDirectory: directory.Path, timeLimit: TimeSpan.FromMinutes(2));
        var first = await Build(); Assert.True(first.ExitCode == 0, first.Output + first.Error);
        VerifySignature(output, rsa); Assert.NotEmpty(AssemblyName.GetAssemblyName(output).GetPublicKeyToken()!);
        var unchanged = await Build(); Assert.True(unchanged.ExitCode == 0, unchanged.Output + unchanged.Error); Assert.DoesNotContain("FPAS: compiling", unchanged.Output);
        var foreign = await ProcessRunner.RunAsync(["build", directory.File("Shared/Shared.csproj"), "-t:Rebuild", "-c", "Release", "-p:SignAssembly=true", "-p:AssemblyOriginatorKeyFile=" + key], workingDirectory: directory.Path);
        Assert.True(foreign.ExitCode == 0, foreign.Output + foreign.Error);
        Assert.Equal(AssemblyName.GetAssemblyName(directory.File("Shared/bin/Release/net10.0/Shared.dll")).GetPublicKey(), AssemblyName.GetAssemblyName(output).GetPublicKey());
        using var second = new RSACryptoServiceProvider(2048); File.WriteAllBytes(key, second.ExportCspBlob(true));
        var rebuilt = await Build(); Assert.True(rebuilt.ExitCode == 0, rebuilt.Output + rebuilt.Error); Assert.Contains("FPAS: compiling", rebuilt.Output); VerifySignature(output, second);
        File.WriteAllBytes(key, AssemblyName.GetAssemblyName(output).GetPublicKey()!);
        var publicSign = await Build("-p:PublicSign=true"); Assert.True(publicSign.ExitCode == 0, publicSign.Output + publicSign.Error);
        CheckReservedSignature(output, signedFlag: true);
        File.WriteAllBytes(key, second.ExportCspBlob(false));
        var delayed = await Build("-p:DelaySign=true"); Assert.True(delayed.ExitCode == 0, delayed.Output + delayed.Error); CheckReservedSignature(output, signedFlag: false);
        File.WriteAllBytes(key, second.ExportCspBlob(true));
        var published = await Cli(directory, "publish", project, "--output", directory.File("signed"), "--mode", "apphost", "--report", "json");
        Assert.True(published.ExitCode == 0, published.Output + published.Error); VerifySignature(directory.File("signed/App.dll"), second);
        Assert.Empty(Directory.GetFiles(directory.File("signed"), "*.snk", SearchOption.AllDirectories));
        File.WriteAllText(key, "invalid signing data");
        var invalid = await Build(); Assert.NotEqual(0, invalid.ExitCode); Assert.Contains("FP4003", invalid.Output + invalid.Error);
    }

    private static void CheckReservedSignature(string file, bool signedFlag)
    {
        using var pe = new PEReader(File.OpenRead(file)); var header = pe.PEHeaders.CorHeader!;
        Assert.Equal(signedFlag, header.Flags.HasFlag(CorFlags.StrongNameSigned));
        Assert.All(pe.GetSectionData(header.StrongNameSignatureDirectory.RelativeVirtualAddress).GetContent(0, header.StrongNameSignatureDirectory.Size), value => Assert.Equal(0, value));
    }

    // Independent ECMA-335 strong-name verification from PE headers and section ranges.
    private static void VerifySignature(string file, RSA rsa)
    {
        var image = File.ReadAllBytes(file);
        using var pe = new PEReader(new MemoryStream(image)); var headers = pe.PEHeaders;
        var directory = headers.CorHeader!.StrongNameSignatureDirectory;
        var section = headers.SectionHeaders.Single(value => directory.RelativeVirtualAddress >= value.VirtualAddress && directory.RelativeVirtualAddress < value.VirtualAddress + value.VirtualSize);
        var offset = section.PointerToRawData + directory.RelativeVirtualAddress - section.VirtualAddress;
        var signature = image.AsSpan(offset, directory.Size).ToArray(); Array.Reverse(signature);
        image.AsSpan(headers.PEHeaderStartOffset + 64, 4).Clear();
        var headerSize = headers.PEHeaderStartOffset + headers.CoffHeader.SizeOfOptionalHeader + headers.SectionHeaders.Length * 40;
        using var hash = IncrementalHash.CreateHash(HashAlgorithmName.SHA1); hash.AppendData(image.AsSpan(0, headerSize));
        foreach (var part in headers.SectionHeaders)
        {
            var start = part.PointerToRawData; var end = start + part.SizeOfRawData;
            if (offset >= start && offset < end)
            { hash.AppendData(image.AsSpan(start, offset - start)); hash.AppendData(image.AsSpan(offset + directory.Size, end - offset - directory.Size)); }
            else hash.AppendData(image.AsSpan(start, part.SizeOfRawData));
        }
        Assert.True(rsa.VerifyHash(hash.GetHashAndReset(), signature, HashAlgorithmName.SHA1, RSASignaturePadding.Pkcs1));
    }
}
