// [Documentation](../../../../docs/architecture/compiler.md#typed-emission)
using System.Diagnostics.CodeAnalysis;
using System.Reflection.Metadata;

namespace Fpas.Compiler.Emission.Metadata;

internal static class WindowsVersionResource
{
    private const ushort Language = 0x0409, Unicode = 1200;
    private const int FixedInfoSize = 52;

    internal static byte[] Create(string assemblyName, WindowsApplicationResources settings)
    {
        if (!TryVersion(settings.FileVersion, out var fileVersion))
            throw InvalidVersion("FileVersion must contain two to four version components from 0 to 65535.");
        var productVersion = TryVersion(settings.ProductVersion.Split('-', '+')[0], out var parsedProduct)
            ? parsedProduct : fileVersion;
        var builder = new BlobBuilder();
        WriteBlock("VS_VERSION_INFO", FixedInfoSize, 0, () =>
        {
            builder.WriteUInt32(0xfeef04bd); // VS_FIXEDFILEINFO signature.
            builder.WriteUInt32(0x00010000); // Structure version.
            WriteVersion(fileVersion);
            WriteVersion(productVersion);
            builder.WriteUInt32(0x3f); // File flags mask.
            builder.WriteUInt32(0); // File flags.
            builder.WriteUInt32(0x00040004); // VOS_NT_WINDOWS32.
            builder.WriteUInt32(1); // VFT_APP.
            builder.WriteBytes(0, 12); // File subtype and date.
            WriteBlock("StringFileInfo", 0, 1, () =>
                WriteBlock("040904b0", 0, 1, () =>
                {
                    WriteString("CompanyName", settings.Company);
                    WriteString("FileDescription", settings.Title);
                    WriteString("FileVersion", settings.FileVersion);
                    WriteString("InternalName", assemblyName);
                    WriteString("LegalCopyright", settings.Copyright);
                    WriteString("OriginalFilename", assemblyName + ".exe");
                    WriteString("ProductName", settings.Product);
                    WriteString("ProductVersion", settings.ProductVersion);
                }));
            WriteBlock("VarFileInfo", 0, 1, () =>
                WriteBlock("Translation", 4, 0, () =>
                {
                    builder.WriteUInt16(Language);
                    builder.WriteUInt16(Unicode);
                }));
        });
        return builder.ToArray();

        void WriteVersion(Version version)
        {
            builder.WriteUInt32((uint)version.Major << 16 | (uint)version.Minor);
            builder.WriteUInt32((uint)Math.Max(0, version.Build) << 16 | (uint)Math.Max(0, version.Revision));
        }

        void WriteString(string name, string value)
        {
            if (value.Length == 0) return;
            if (value.Length >= ushort.MaxValue || value.Contains('\0'))
                throw InvalidVersion($"Invalid Windows version metadata value for {name}.");
            WriteBlock(name, value.Length + 1, 1, () => WriteText(value));
        }

        void WriteBlock(string key, int valueLength, ushort type, Action write)
        {
            builder.Align(4);
            int start = builder.Count;
            var length = builder.ReserveBytes(sizeof(ushort));
            builder.WriteUInt16((ushort)valueLength);
            builder.WriteUInt16(type);
            WriteText(key);
            builder.Align(4);
            write();
            if (builder.Count - start > ushort.MaxValue)
                throw InvalidVersion("Windows version metadata exceeds its format's size limit.");
            new BlobWriter(length).WriteUInt16((ushort)(builder.Count - start));
        }

        void WriteText(string value)
        {
            builder.WriteUTF16(value);
            builder.WriteUInt16(0);
        }
    }

    private static bool TryVersion(string value, [NotNullWhen(true)] out Version? version) =>
        Version.TryParse(value, out version) && version.Major <= ushort.MaxValue && version.Minor <= ushort.MaxValue &&
        version.Build <= ushort.MaxValue && version.Revision <= ushort.MaxValue;

    private static CompilationFailure InvalidVersion(string message) => new(new("FP4005", message, null));
}
