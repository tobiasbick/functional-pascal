using Fpas.Compiler.Emission.Debugging;
using Fpas.Compiler.Emission.Bodies;
using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Analysis;
using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;
using System.Security.Cryptography;
using System.Text;

namespace Fpas.Compiler.Emission.Metadata;

internal sealed partial class AssemblyEmitter
{
    private readonly MetadataBuilder metadata = new();
    private readonly Dictionary<string, AssemblyReferenceHandle> assemblies = new(StringComparer.Ordinal);
    private readonly Dictionary<ImportedType, TypeReferenceHandle> types = [];
    private readonly Dictionary<string, MemberReferenceHandle> methods = new(StringComparer.Ordinal);
    private readonly Dictionary<string, MethodSpecificationHandle> methodSpecs = new(StringComparer.Ordinal);
    private AssemblyLayout layout = null!;
    private MemberReferenceHandle nullableConstructor;
    private MemberReferenceHandle nullableAnnotationConstructor;
    private ReferenceCatalog catalog = null!;
    private readonly Dictionary<(TypeSymbol Type, IReadOnlyList<GenericParameterSymbol>? Context), EntityHandle> typeHandles = [];
    private readonly Dictionary<string, MemberReferenceHandle> fields = [];
    private PortableDebugInfo? debug;

    internal static CompiledProgram Emit(PreparedCompilation prepared, string assemblyName, bool emitDebug, bool optimize)
        => new AssemblyEmitter().Build(prepared, assemblyName, emitDebug, optimize);

    private CompiledProgram Build(PreparedCompilation prepared, string assemblyName, bool emitDebug, bool optimize)
    {
        var program = prepared.Program;
        var sources = prepared.Catalog.Units.Select(unit => unit.Source);
        catalog = prepared.References;
        sourceAssemblyName = assemblyName;
        layout = new(program, prepared.AsyncPlans);
        if (emitDebug) debug = new(sources);
        using var sourceHash = IncrementalHash.CreateHash(HashAlgorithmName.SHA256);
        sourceHash.AppendData(Encoding.UTF8.GetBytes(assemblyName));
        sourceHash.AppendData([(byte)(emitDebug ? 1 : 0), (byte)(optimize ? 1 : 0)]);
        foreach (var text in sources)
            sourceHash.AppendData(SHA256.HashData(Encoding.UTF8.GetBytes(text.Text)));
        var mvid = new Guid(sourceHash.GetHashAndReset()[..16]);
        metadata.AddModule(0, metadata.GetOrAddString(assemblyName + ".dll"), metadata.GetOrAddGuid(mvid), default, default);
        metadata.AddAssembly(metadata.GetOrAddString(assemblyName), new Version(1, 0, 0, 0), default, default, 0, AssemblyHashAlgorithm.None);
        AddDebuggableAttribute(optimize);
        var objectType = metadata.AddTypeReference(Reference(program.Runtime), metadata.GetOrAddString("System"), metadata.GetOrAddString("Object"));
        var bodies = new BlobBuilder();
        var bodyEncoder = new MethodBodyStreamEncoder(bodies);
        MethodDefinitionHandle entry = default;
        var parameterCount = 0;
        var methodCount = 0;
        metadata.AddTypeDefinition(TypeAttributes.NotPublic, default, metadata.GetOrAddString("<Module>"), default,
            MetadataTokens.FieldDefinitionHandle(1), MetadataTokens.MethodDefinitionHandle(1));
        foreach (var unit in program.Units)
        {
            var separator = unit.Name.LastIndexOf('.');
            var ns = separator < 0 ? "" : unit.Name[..separator];
            var name = unit.Name[(separator + 1)..];
            var container = metadata.AddTypeDefinition(TypeAttributes.Public | TypeAttributes.Abstract | TypeAttributes.Sealed,
                metadata.GetOrAddString(ns), metadata.GetOrAddString(name), objectType,
                MetadataTokens.FieldDefinitionHandle(metadata.GetRowCount(TableIndex.Field) + 1), MetadataTokens.MethodDefinitionHandle(methodCount + 1));
            AddNullabilityContext(container, program.Runtime);
            EmitGlobalFields(unit, program.Runtime);
            foreach (var routine in unit.Routines)
            {
                var firstParameter = AddParameters(routine.Symbol, ref parameterCount, program.Runtime);
                var signature = MethodSignature(routine.Symbol.Result, routine.Symbol.Parameters.Select(parameter => parameter.SignatureType).ToArray(), genericArity: routine.Symbol.GenericParameters.Count);
                var attributes = MethodAttributes.Static | MethodAttributes.HideBySig |
                    (routine.Symbol.IsPublic ? MethodAttributes.Public : MethodAttributes.Private);
                if (routine.Symbol.Name == ".cctor") attributes |= MethodAttributes.SpecialName | MethodAttributes.RTSpecialName;
                var handle = metadata.AddMethodDefinition(attributes, SourceMethodFlags(routine.Attributes),
                    metadata.GetOrAddString(routine.Symbol.Name), signature, EmitBody(routine, bodyEncoder), firstParameter);
                if (routine.Symbol.IsAsync) AddAsyncAttribute(handle, layout.AsyncRoutines[routine], routine.Attributes);
                AddSourceAttributes(handle, routine.Attributes);
                RegisterGenerics(handle, routine.Symbol.GenericParameters);
                methodCount++;
                if (routine.Symbol.IsEntryPoint && !routine.Symbol.IsAsync)
                    entry = handle;
            }
            if (layout.EntryBridges.TryGetValue(unit.Name, out var bridge))
            {
                EmitEntryBridge(unit.Routines.Single(routine => routine.Symbol.IsEntryPoint), bridge, bodyEncoder, ref parameterCount, program.Runtime);
                entry = bridge; methodCount++;
            }
            foreach (var alias in unit.Aliases)
            {
                EmitAlias(alias, bodyEncoder, ref parameterCount, program.Runtime); methodCount++;
            }
        }

        EmitUnions(program, bodyEncoder, ref parameterCount, objectType);
        EmitRecords(program, bodyEncoder, ref parameterCount, objectType);
        EmitInterfaces(program.Runtime, ref parameterCount);
        EmitClosures(program, bodyEncoder, ref parameterCount, objectType);
        EmitAsyncMachines(bodyEncoder, ref parameterCount, objectType, program.Runtime);
        EmitScalars(bodyEncoder, ref parameterCount, program.Runtime);
        EmitDebuggerProxies(bodyEncoder, ref parameterCount, objectType, program.Runtime);
        EmitGenericParameters(program.Runtime);
        var symbols = debug?.Build(metadata, entry);
        DebugDirectoryBuilder? directory = null;
        if (symbols is not null)
        {
            directory = new();
            directory.AddCodeViewEntry(assemblyName + ".pdb", symbols.Value.Id, 0x0100);
            directory.AddPdbChecksumEntry("SHA256", System.Collections.Immutable.ImmutableArray.Create(SHA256.HashData(symbols.Value.Image)));
            directory.AddReproducibleEntry();
        }
        var pe = new ManagedPEBuilder(
            new PEHeaderBuilder(imageCharacteristics: Characteristics.ExecutableImage | Characteristics.Dll),
            new MetadataRootBuilder(metadata), bodies, debugDirectoryBuilder: directory, entryPoint: entry, flags: CorFlags.ILOnly,
            deterministicIdProvider: HashContent);
        var image = new BlobBuilder();
        pe.Serialize(image);
        return new(image.ToArray(), program.IsExecutable ? """
            {"runtimeOptions":{"tfm":"net10.0","framework":{"name":"Microsoft.NETCore.App","version":"10.0.0"}}}
            """ + "\n" : null, symbols?.Image);
    }

    private int EmitBody(BoundRoutine routine, MethodBodyStreamEncoder bodies)
    {
        if (routine.Symbol.IsAsync) return EmitAsyncStart(layout.AsyncRoutines[routine], bodies);
        var emitter = new BodyEmitter(Reference, FieldReference, TypeHandle, catalog, metadata.GetOrAddUserString, debug is not null);
        var body = emitter.Build(routine);
        StandaloneSignatureHandle locals = default;
        if (body.Locals.Count != 0)
        {
            var signature = new BlobBuilder();
            var variables = new BlobEncoder(signature).LocalVariableSignature(body.Locals.Count);
            foreach (var type in body.Locals)
                EncodeType(variables.AddVariable().Type(isByRef: type.Kind == TypeKind.ByRef), type.Kind == TypeKind.ByRef ? type.Element! : type);
            locals = metadata.AddStandaloneSignature(metadata.GetOrAddBlob(signature));
        }
        var handle = layout.SourceMethods.TryGetValue(routine.Symbol, out var sourceMethod) ? sourceMethod :
                layout.ClosureInvocations[layout.Closures.Single(closure => ReferenceEquals(closure.Body, routine))];
        debug?.AddMethod(handle, emitter.SequencePoints, routine.Locals, locals, body.Code.Offset, emitter.LocalScopes);
        return bodies.AddMethodBody(body.Code, body.MaxStack, locals);
    }

    private void AddNullabilityContext(TypeDefinitionHandle container, AssemblyIdentity runtime)
    {
        if (nullableConstructor.IsNil)
        {
            var attribute = metadata.AddTypeReference(Reference(runtime), metadata.GetOrAddString("System.Runtime.CompilerServices"),
                metadata.GetOrAddString("NullableContextAttribute"));
            var signature = new BlobBuilder();
            new BlobEncoder(signature).MethodSignature(isInstanceMethod: true).Parameters(1, result => result.Void(), parameters => parameters.AddParameter().Type().Byte());
            nullableConstructor = metadata.AddMemberReference(attribute, metadata.GetOrAddString(".ctor"), metadata.GetOrAddBlob(signature));
        }
        metadata.AddCustomAttribute(container, nullableConstructor, metadata.GetOrAddBlob(new byte[] { 1, 0, 1, 0, 0 }));
    }

    private AssemblyReferenceHandle Reference(AssemblyIdentity identity)
    {
        if (!assemblies.TryGetValue(identity.Name, out var handle))
        {
            handle = metadata.AddAssemblyReference(metadata.GetOrAddString(identity.Name), identity.Version,
                metadata.GetOrAddString(identity.Culture), metadata.GetOrAddBlob(identity.PublicKeyToken), identity.Flags, default);
            assemblies.Add(identity.Name, handle);
        }
        return handle;
    }

    private static BlobContentId HashContent(IEnumerable<Blob> blobs)
    {
        using var hash = IncrementalHash.CreateHash(HashAlgorithmName.SHA256);
        foreach (var blob in blobs)
            hash.AppendData(blob.GetBytes());
        return BlobContentId.FromHash(hash.GetHashAndReset());
    }
}
