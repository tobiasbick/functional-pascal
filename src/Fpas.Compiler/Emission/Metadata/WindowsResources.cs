// [Documentation](../../../../docs/architecture/compiler.md#typed-emission)
using System.Buffers.Binary;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;

namespace Fpas.Compiler.Emission.Metadata;

internal sealed class WindowsResources : ResourceSectionBuilder
{
    private const int IconType = 3, GroupIconType = 14, VersionType = 16;
    private const int DirectoryHeaderSize = 16, DirectoryEntrySize = 8, DataEntrySize = 16;
    private const uint DirectoryFlag = 0x80000000;
    private readonly Resource[] entries;

    internal WindowsResources(string assemblyName, WindowsApplicationResources settings)
    {
        var resources = new List<Resource>();
        if (settings.Icon is { } icon) AddIcons(icon, resources);
        resources.Add(new(VersionType, 1, WindowsVersionResource.Create(assemblyName, settings)));
        entries = resources.ToArray();
    }

    private static void AddIcons(byte[] icon, List<Resource> resources)
    {
        const int headerSize = 6, entrySize = 16;
        if (icon.Length < headerSize || ReadUInt16(icon) != 0 || ReadUInt16(icon.AsSpan(2)) != 1)
            throw InvalidIcon();
        int count = ReadUInt16(icon.AsSpan(4));
        int imagesStart = headerSize + count * entrySize;
        if (count == 0 || imagesStart > icon.Length) throw InvalidIcon();

        var group = new BlobBuilder();
        group.WriteBytes(icon, 0, headerSize);
        for (int index = 0; index < count; index++)
        {
            int entry = headerSize + index * entrySize;
            int size = BinaryPrimitives.ReadInt32LittleEndian(icon.AsSpan(entry + 8));
            int offset = BinaryPrimitives.ReadInt32LittleEndian(icon.AsSpan(entry + 12));
            if (offset < imagesStart || offset > icon.Length || size <= 0 || size > icon.Length - offset)
                throw InvalidIcon();

            int id = index + 1;
            resources.Add(new(IconType, id, icon, offset, size));
            group.WriteBytes(icon, entry, 12);
            group.WriteUInt16((ushort)id);
        }
        resources.Add(new(GroupIconType, 32512, group.ToArray()));
    }

    protected override void Serialize(BlobBuilder builder, SectionLocation location)
    {
        var types = entries.GroupBy(entry => entry.Type).ToArray();
        int typeOffset = DirectoryHeaderSize + DirectoryEntrySize * types.Length;
        WriteDirectory(builder, types.Length);
        foreach (var type in types)
        {
            WriteEntry(builder, type.Key, (uint)typeOffset | DirectoryFlag);
            typeOffset += DirectoryHeaderSize + DirectoryEntrySize * type.Count();
        }

        int languageOffset = typeOffset;
        foreach (var type in types)
        {
            WriteDirectory(builder, type.Count());
            foreach (var entry in type)
            {
                WriteEntry(builder, entry.Id, (uint)languageOffset | DirectoryFlag);
                languageOffset += DirectoryHeaderSize + DirectoryEntrySize;
            }
        }

        int dataOffset = languageOffset;
        foreach (var entry in entries)
        {
            WriteDirectory(builder, 1);
            WriteEntry(builder, 0, (uint)dataOffset);
            dataOffset += DataEntrySize;
        }

        int payloadOffset = dataOffset;
        foreach (var entry in entries)
        {
            builder.WriteInt32(checked(location.RelativeVirtualAddress + payloadOffset));
            builder.WriteInt32(entry.Length);
            builder.WriteUInt32(0); // Code page.
            builder.WriteUInt32(0); // Reserved.
            payloadOffset = checked((payloadOffset + entry.Length + 3) & ~3);
        }
        foreach (var entry in entries)
        {
            builder.WriteBytes(entry.Bytes, entry.Offset, entry.Length);
            builder.Align(4);
        }
    }

    private static void WriteDirectory(BlobBuilder builder, int entryCount)
    {
        builder.WriteBytes(0, DirectoryHeaderSize - sizeof(ushort));
        builder.WriteUInt16((ushort)entryCount);
    }

    private static void WriteEntry(BlobBuilder builder, int id, uint offset)
    {
        builder.WriteInt32(id);
        builder.WriteUInt32(offset);
    }

    private static ushort ReadUInt16(ReadOnlySpan<byte> bytes) => BinaryPrimitives.ReadUInt16LittleEndian(bytes);

    private static CompilationFailure InvalidIcon() =>
        new(new("FP4005", "ApplicationIcon must contain a valid Windows .ico directory and image data.", null));

    private sealed record Resource(int Type, int Id, byte[] Bytes, int Offset, int Length)
    {
        internal Resource(int type, int id, byte[] bytes) : this(type, id, bytes, 0, bytes.Length) { }
    }
}
