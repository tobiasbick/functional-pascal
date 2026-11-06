using System.Runtime.InteropServices;
using Fpas.Debugging.Core;
using Fpas.Debugging.Symbols;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession
{
    private static readonly Guid referenceValueId = new("CC7BCAF9-8A68-11D2-983C-0000F808342D");
    private static readonly Guid objectValueId = new("18AD3D6E-B7D2-11D2-BD04-0000F80849BD");
    private static readonly Guid value2Id = new("5E0B54E7-D88A-4626-9420-A691E0A78B49");
    private static readonly Guid stringValueId = new("CC7BCAFD-8A68-11D2-983C-0000F808342D");
    private static readonly Guid boxValueId = new("CC7BCAFC-8A68-11D2-983C-0000F808342D");
    private readonly Dictionary<(string Path, Guid Identity), MetadataImage> metadataImages = [];

    private MetadataImage MetadataFor(NativeObject module)
    {
        var path = module.Name(6); var identity = module.ModuleIdentity();
        if (catalog.Probe(path, identity).Symbols is { } symbols) return symbols;
        if (metadataImages.TryGetValue((path, identity), out var existing)) return existing;
        if (metadataImages.Count >= 1024) throw new DebugException("limit_exceeded", "The runtime metadata catalog is full.");
        var image = new MetadataImage(path);
        if (image.Identity != identity) { image.Dispose(); throw new DebugException("module_mismatch", "Runtime metadata does not match the loaded module."); }
        metadataImages.Add((path, identity), image); return image;
    }

    private static NativeObject ObjectValue(NativeObject value)
    {
        using var reference = value.TryQuery(referenceValueId);
        if (reference is null) return value.Query(objectValueId);
        if (reference.UInt32(7) != 0) throw new DebugException("unavailable", "The object reference is null.");
        using var contents = reference.Object(10);
        using var box = contents.TryQuery(boxValueId);
        if (box is not null) return box.Object(9);
        return contents.Query(objectValueId);
    }

    private static NativeObject ExactType(NativeObject value)
    {
        using var extended = value.Query(value2Id);
        return extended.Object(3);
    }

    private unsafe PassiveText StringValue(NativeObject value)
    {
        using var reference = value.Query(referenceValueId);
        if (reference.UInt32(7) != 0) return new(null);
        using var contents = reference.Object(10);
        using var text = contents.Query(stringValueId);
        var length = text.UInt32(9);
        var buffer = new char[Math.Min(length, 8192) + 1]; uint written = 0;
        fixed (char* pointer = buffer)
            NativeObject.Check(((delegate* unmanaged[Stdcall]<nint, uint, uint*, char*, int>)text.Slot(10))(text.Pointer, (uint)buffer.Length, &written, pointer));
        var count = Math.Min((int)Math.Min(length, written), buffer.Length - 1);
        return Values.StoredText.Read(buffer.AsSpan(0, count), length);
    }

    private void ReleaseMetadata()
    {
        foreach (var metadata in metadataImages.Values) metadata.Dispose(); metadataImages.Clear();
    }
}
