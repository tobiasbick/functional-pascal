using System.Reflection.Metadata;

namespace Fpas.Debugging.Symbols;

internal sealed record SourceStorage(string Name, bool Cell);

internal sealed partial class ModuleSymbols
{
    internal SourceStorage? Storage(EntityHandle parent)
    {
        try { return ReadStorage(parent); }
        catch (BadImageFormatException) { throw new Core.DebugException("invalid_metadata", "Source storage information is invalid or unsupported."); }
    }

    private SourceStorage? ReadStorage(EntityHandle parent)
    {
        SourceStorage? result = null;
        foreach (var handle in Pdb.GetCustomDebugInformation(parent))
        {
            var custom = Pdb.GetCustomDebugInformation(handle);
            if (Pdb.GetGuid(custom.Kind) != new Guid("4C590C68-DC5F-4A9D-89A5-B8CE5B937137")) continue;
            if (result is not null) throw new BadImageFormatException("Duplicate source storage information.");
            var blob = Pdb.GetBlobReader(custom.Value);
            if (blob.Length > 32778 || blob.ReadUInt32() != 1) throw new BadImageFormatException("Unsupported source storage information.");
            var flags = blob.ReadByte(); var name = blob.ReadSerializedString();
            if (flags > 1 || string.IsNullOrEmpty(name) || name.Length > 8192 || blob.RemainingBytes != 0) throw new BadImageFormatException("Invalid source storage information.");
            result = new(name, flags == 1);
        }
        return result;
    }
}
