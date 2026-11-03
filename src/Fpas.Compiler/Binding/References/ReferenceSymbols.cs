using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using System.Reflection;
using System.Reflection.Metadata;

namespace Fpas.Compiler.Binding.References;

internal sealed record AssemblyIdentity(string Name, Version Version, string Culture, byte[] PublicKeyToken,
    AssemblyFlags Flags);

internal sealed record ImportedType(ReferenceAssembly Assembly, TypeDefinitionHandle Handle, string Namespace, string Name)
{
    internal ImportedType? DeclaringType
    {
        get
        {
            var handle = Definition.GetDeclaringType();
            if (handle.IsNil) return null;
            var parent = Assembly.Reader.GetTypeDefinition(handle);
            return new(Assembly, handle, Assembly.Reader.GetString(parent.Namespace), Assembly.Reader.GetString(parent.Name));
        }
    }
    internal bool IsPublic => DeclaringType is { } parent
        ? (Definition.Attributes & TypeAttributes.VisibilityMask) == TypeAttributes.NestedPublic && parent.IsPublic
        : (Definition.Attributes & TypeAttributes.VisibilityMask) == TypeAttributes.Public;
    internal string FullName => DeclaringType is { } parent ? parent.FullName + "+" + Name : Namespace.Length == 0 ? Name : Namespace + "." + Name;
    internal TypeDefinition Definition => Assembly.Reader.GetTypeDefinition(Handle);
    internal bool IsValueType => FullName != "System.Enum" && (BaseName is "System.ValueType" or "System.Enum");
    internal bool IsEnum => BaseName == "System.Enum";
    private string BaseName
    {
        get
        {
            var handle = Definition.BaseType;
            if (handle.IsNil) return "";
            if (handle.Kind == HandleKind.TypeReference)
            {
                var type = Assembly.Reader.GetTypeReference((TypeReferenceHandle)handle);
                return Assembly.Reader.GetString(type.Namespace) + "." + Assembly.Reader.GetString(type.Name);
            }
            if (handle.Kind == HandleKind.TypeDefinition)
            {
                var type = Assembly.Reader.GetTypeDefinition((TypeDefinitionHandle)handle);
                return Assembly.Reader.GetString(type.Namespace) + "." + Assembly.Reader.GetString(type.Name);
            }
            return "";
        }
    }
}
internal sealed record ImportedMethod(TypeSymbol Owner, string Name, TypeSymbol Result, IReadOnlyList<TypeSymbol> Parameters,
    bool IsStatic, bool IsVirtual = false, TypeSymbol? SignatureResult = null, IReadOnlyList<TypeSymbol>? SignatureParameters = null, bool ReturnsByRef = false, TypeList? MethodArguments = null, MethodDefinitionHandle DefinitionHandle = default, RoutineSymbol? SourceMethod = null, IReadOnlyList<ParameterSymbol>? ParameterInfo = null, TypeSymbol? ConstrainedType = null)
{
    internal ParameterSymbol Parameter(int index) => ParameterInfo?[index] ?? new("", Parameters[index], index);
    internal bool Flexible => ParameterInfo?.Any(parameter => parameter.Mode != "" || parameter.Default is not null) == true;
    internal IReadOnlyList<TypeSymbol> DelegateParameters => Parameters.Select((type, index) => Parameter(index).ByRef ? TypeSymbol.ByRef(type) : type).ToArray();
}
internal sealed record ImportedField(TypeSymbol Owner, string Name, TypeSymbol Type, bool IsStatic, bool IsReadOnly, object? Constant = null, bool IsLiteral = false, TypeSymbol? SignatureType = null);
internal sealed record ImportedProperty(TypeSymbol Type, ImportedMethod? Getter, ImportedMethod? Setter);
