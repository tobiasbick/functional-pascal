// [Documentation](../../../docs/architecture/debugger-core.md#passive-scalar-bindings)
using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using Fpas.Debugging.Core;
using Fpas.Debugging.Symbols;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession
{
    private sealed partial class EvaluationContext
    {
        private readonly Dictionary<string, DebugBinding?> nameTypes = new(StringComparer.Ordinal);

        public DebugBinding? NameType(string name)
        {
            if (nameTypes.TryGetValue(name, out var existing)) return existing;
            var context = session.frames[frame]; var symbols = context.Symbols;
            if (symbols is null || context.Data.IlOffset is not { } offset)
                throw new DebugException("unavailable", "No lexical context is available for this frame.");
            using var function = context.Frame.Object(5); using var module = function.Object(3);
            using var il = context.Frame.Query(ilFrameId);
            var receipt = session.modules.Single(item => item.Native.SameIdentity(module.Pointer));
            using var ownerType = session.FrameOwnerType(context.Frame);
            var remaining = 128; var identity = session.DescribeType(ownerType, 0, ref remaining);
            var token = context.Data.MethodToken; DebugBinding? value = null;
            var signature = new StoredTypeSignature(symbols, receipt.Id, identity.Arguments ?? [],
                MethodArguments(context.Frame, identity.Arguments?.Length ?? 0,
                    symbols.Metadata.GetMethodDefinition((MethodDefinitionHandle)MetadataTokens.Handle((int)token)).GetGenericParameters().Count));
            if (symbols.AsyncMethod(token) is { MoveNext: var moveNext } && moveNext == token)
            {
                foreach (var field in symbols.AsyncBindings(token, offset))
                {
                    Read(1);
                    if (field.Name == name) { value = StorageType(() => AsyncStorage(field.Token), () => signature.Field(field.Token), field.Cell); break; }
                }
            }
            else
            {
                foreach (var local in symbols.Locals(token, offset))
                {
                    Read(1);
                    if (local.Name == name) { value = StorageType(() => il.Object(14, (uint)local.Slot), () => signature.Local(token, local.Slot), local.Cell); break; }
                }
                if (value is null)
                    foreach (var argument in symbols.Arguments(token, "source"))
                    {
                        Read(1);
                        if (argument.Name != name) continue;
                        var definition = symbols.Metadata.GetMethodDefinition((MethodDefinitionHandle)MetadataTokens.Handle((int)token));
                        var instance = (definition.Attributes & MethodAttributes.Static) == 0;
                        value = StorageType(() => il.Object(16, (uint)argument.Index),
                            () => instance && argument.Index == 0 ? identity : signature.Argument(token, argument.Index - (instance ? 1 : 0)), false);
                        break;
                    }
            }
            if (value is null)
            {
                var sourceToken = symbols.AsyncMethod(token)?.Kickoff ?? token;
                var method = symbols.Metadata.GetMethodDefinition((MethodDefinitionHandle)MetadataTokens.Handle((int)sourceToken));
                var owner = method.GetDeclaringType();
                if (sourceToken != token && name == "self" && symbols.SourceReceiver(sourceToken))
                {
                    var machine = symbols.Metadata.GetMethodDefinition((MethodDefinitionHandle)MetadataTokens.Handle((int)token)).GetDeclaringType();
                    var receiver = symbols.Fields((uint)MetadataTokens.GetToken(machine)).FirstOrDefault(field => field.Name == "<>4__this");
                    if (receiver.Token != 0) value = StorageType(() => AsyncStorage(receiver.Token), () => signature.Field(receiver.Token), false);
                }
                if (value is null && (method.Attributes & MethodAttributes.Static) == 0)
                    foreach (var field in symbols.Fields((uint)MetadataTokens.GetToken(owner)))
                    {
                        Read(1); var storage = symbols.Storage(MetadataTokens.EntityHandle((int)field.Token));
                        if (storage?.Name == name)
                        {
                            value = StorageType(() => CaptureStorage(field.Token), () => signature.Field(field.Token), storage.Cell); break;
                        }
                    }
                for (var depth = 0; value is null && !owner.IsNil; depth++)
                {
                    if (depth == 64) throw new DebugException("limit_exceeded", "Static source scopes exceed 64 declaring types.");
                    MetadataRead(); using var runtimeClass = module.Object(11, (uint)MetadataTokens.GetToken(owner));
                    using var type = session.DefinitionType(runtimeClass, context.Frame);
                    value = StaticFieldType(type, name);
                    owner = symbols.Metadata.GetTypeDefinition(owner).GetDeclaringType();
                }
            }
            nameTypes.Add(name, value); return value;

            NativeObject AsyncStorage(uint field)
            {
                using var receiver = il.Object(16, 0); using var instance = ObjectValue(receiver); using var owner = instance.Object(7);
                return instance.Field(owner, field);
            }

            NativeObject CaptureStorage(uint field)
            {
                using var receiver = SourceReceiverValue(); using var instance = ObjectValue(receiver); using var owner = instance.Object(7);
                return instance.Field(owner, field);
            }

            NativeObject SourceReceiverValue()
            {
                if (symbols.AsyncMethod(token)?.MoveNext != token) return il.Object(16, 0);
                var machine = symbols.Metadata.GetMethodDefinition((MethodDefinitionHandle)MetadataTokens.Handle((int)token)).GetDeclaringType();
                var receiver = symbols.Fields((uint)MetadataTokens.GetToken(machine)).FirstOrDefault(field => field.Name == "<>4__this");
                if (receiver.Token == 0) throw new DebugException("unavailable", "The source receiver's declared storage is unavailable.");
                return AsyncStorage(receiver.Token);
            }
        }

        private DebugTypeIdentity[] MethodArguments(NativeObject selected, int ownerCount, int count)
        {
            if (count == 0) return [];
            if (count > 64 || ownerCount > 64) throw new DebugException("limit_exceeded", "The frame has more than 64 generic arguments.");
            using var extended = selected.Query(ilFrame2Id); using var enumeration = extended.Object(4);
            var result = new List<DebugTypeIdentity>();
            for (var index = 0; index < ownerCount + count; index++)
            {
                using var argument = enumeration.Next() ?? throw new DebugException("unavailable", "The frame's generic arguments are unavailable.");
                if (index < ownerCount) continue;
                var remaining = 128; result.Add(session.DescribeType(argument, 0, ref remaining));
            }
            return result.ToArray();
        }

        private DebugBinding StorageType(Func<NativeObject> storage, Func<DebugTypeIdentity> declared, bool cell)
        {
            try
            {
                using var value = storage();
                return CellType(session.ValueTypeIdentity(value), cell);
            }
            catch (NativeOperationException exception) when (exception.Result is unchecked((int)0x80131304)
                or unchecked((int)0x80131306) or unchecked((int)0x80131334))
            { return CellType(declared(), cell); }
        }

        private DebugBinding? StaticFieldType(NativeObject initial, string name)
        {
            var type = NativeObject.Borrow(initial.Pointer);
            try
            {
                for (var depth = 0; depth < 64; depth++)
                {
                    MetadataRead(); using var owner = type.Object(4); using var module = owner.Object(3);
                    var image = session.MetadataFor(module);
                    var receipt = session.modules.Single(item => item.Native.SameIdentity(module.Pointer));
                    foreach (var field in image.Fields(owner.UInt32(4)))
                    {
                        Read(1);
                        var definition = image.Metadata.GetFieldDefinition((FieldDefinitionHandle)MetadataTokens.Handle((int)field.Token));
                        if (field.Name != name || (definition.Attributes & FieldAttributes.Static) == 0) continue;
                        if ((definition.Attributes & FieldAttributes.Literal) != 0) return ConstantBinding(name, image, definition) with { Constant = true };
                        var remaining = 128; var identity = session.DescribeType(type, 0, ref remaining);
                        return FieldType(new(image, receipt.Id, identity.Arguments ?? []), field.Token, false);
                    }
                    var next = type.OptionalObject(7);
                    if (next is null) return null;
                    type.Dispose(); type = next;
                }
                throw new DebugException("limit_exceeded", "Static field type lookup exceeds 64 base types.");
            }
            finally { type.Dispose(); }
        }

        private static DebugBinding FieldType(StoredTypeSignature signature, uint field, bool cell)
            => CellType(signature.Field(field), cell);

        private static DebugBinding CellType(DebugTypeIdentity type, bool cell)
            => TypeValue(!cell ? type : type.Arguments is [var element] ? element
                : throw new DebugException("unavailable", "The compiler capture cell's declared type is unavailable."));
    }
}
