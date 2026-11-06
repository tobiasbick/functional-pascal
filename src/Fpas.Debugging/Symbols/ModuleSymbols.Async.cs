using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;

namespace Fpas.Debugging.Symbols;

internal sealed record AwaitPoint(int Yield, uint ResumeMethod, int Resume);
internal sealed record HoistedScope(int Start, int Length);
internal sealed record AsyncMethodInfo(uint MoveNext, uint Kickoff, AwaitPoint[] Awaits, HoistedScope[] Scopes);

internal sealed partial class ModuleSymbols
{
    private Dictionary<uint, AsyncMethodInfo>? asyncMethods;
    internal IReadOnlyCollection<AsyncMethodInfo> AsyncMethods { get { _ = AsyncMethod(0); return asyncMethods!.Values; } }

    internal AsyncMethodInfo? AsyncType(uint type) => AsyncMethods.FirstOrDefault(info =>
        MetadataTokens.GetToken(Metadata.GetMethodDefinition((MethodDefinitionHandle)MetadataTokens.Handle((int)info.MoveNext)).GetDeclaringType()) == type);

    internal AsyncMethodInfo? AsyncMethod(uint token)
    {
        if (asyncMethods is null)
        {
            var parsed = new Dictionary<uint, AsyncMethodInfo>();
            foreach (var handle in Pdb.MethodDebugInformation)
            {
                var kickoff = Pdb.GetMethodDebugInformation(handle).GetStateMachineKickoffMethod();
                if (kickoff.IsNil) continue;
                var method = handle.ToDefinitionHandle(); var awaits = new List<AwaitPoint>(); var scopes = new List<HoistedScope>();
                foreach (var custom in Pdb.GetCustomDebugInformation(method))
                {
                    var data = Pdb.GetCustomDebugInformation(custom); var kind = Pdb.GetGuid(data.Kind); var blob = Pdb.GetBlobReader(data.Value);
                    if (kind == new Guid("54FD2AC5-E925-401A-9C2A-F94F171072F8"))
                    {
                        _ = blob.ReadUInt32();
                        while (blob.RemainingBytes != 0)
                        {
                            if (awaits.Count >= 4096) throw new BadImageFormatException("Async stepping metadata exceeds its await limit.");
                            var yield = blob.ReadInt32(); var resume = blob.ReadInt32(); var row = blob.ReadCompressedInteger();
                            if (yield < 0 || resume < 0 || row <= 0 || row > Metadata.MethodDefinitions.Count) throw new BadImageFormatException("Invalid async resume metadata.");
                            awaits.Add(new(yield, (uint)MetadataTokens.GetToken(MetadataTokens.MethodDefinitionHandle(row)), resume));
                        }
                    }
                    else if (kind == new Guid("6DA9A61E-F8C7-4874-BE62-68BC5630DF71"))
                    {
                        while (blob.RemainingBytes != 0)
                        {
                            if (scopes.Count >= 4096) throw new BadImageFormatException("Async hoisted scope metadata exceeds its limit.");
                            scopes.Add(new(blob.ReadInt32(), blob.ReadInt32()));
                        }
                    }
                }
                var info = new AsyncMethodInfo((uint)MetadataTokens.GetToken(method), (uint)MetadataTokens.GetToken(kickoff), awaits.ToArray(), scopes.ToArray());
                parsed.Add(info.MoveNext, info);
            }
            asyncMethods = parsed;
        }
        return asyncMethods.GetValueOrDefault(token) ?? asyncMethods.Values.FirstOrDefault(info => info.Kickoff == token);
    }

    internal (string Name, uint Token, bool Cell)[] AsyncBindings(uint token, int offset, string view = "source")
    {
        var info = AsyncMethod(token);
        if (info is null || info.MoveNext != token) return [];
        var type = Metadata.GetMethodDefinition((MethodDefinitionHandle)MetadataTokens.Handle((int)token)).GetDeclaringType();
        var result = new List<(string Name, uint Token, bool Cell, int Scope)>();
        foreach (var field in Fields((uint)MetadataTokens.GetToken(type)))
        {
            if (!field.Name.StartsWith('<')) { result.Add((field.Name, field.Token, false, int.MaxValue)); continue; }
            var delimiter = field.Name.IndexOf(">5__", StringComparison.Ordinal);
            if (delimiter <= 1 || !int.TryParse(field.Name.AsSpan(delimiter + 4), out var ordinal) || ordinal <= 0 || ordinal > info.Scopes.Length) continue;
            var scope = info.Scopes[ordinal - 1];
            if (scope.Length <= 0 || offset < scope.Start || (long)offset >= (long)scope.Start + scope.Length) continue;
            var storage = view == "source" ? Storage(MetadataTokens.EntityHandle((int)field.Token)) : null;
            result.Add((storage?.Name ?? field.Name[1..delimiter], field.Token, storage?.Cell == true, scope.Length));
        }
        return result.OrderBy(item => item.Scope).DistinctBy(item => item.Name).Select(item => (item.Name, item.Token, item.Cell)).ToArray();
    }
}
