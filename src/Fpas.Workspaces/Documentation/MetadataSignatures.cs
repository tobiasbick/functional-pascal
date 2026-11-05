using System.Reflection;
using System.Reflection.Metadata;
using System.Globalization;
using System.Collections.Immutable;

namespace Fpas.Workspaces.Documentation;

internal sealed class MetadataSignatures(MetadataReader reader, TypeDefinitionHandle owner)
{
    private readonly MetadataSignatureTypes types = new();
    internal string OwnerName { get; } = MetadataSignatureTypes.DefinitionName(reader, owner);
    private GenericNames Names(MethodDefinitionHandle method = default) => new(
        reader.GetTypeDefinition(owner).GetGenericParameters().Select(handle => reader.GetString(reader.GetGenericParameter(handle).Name)).ToArray(),
        method.IsNil ? [] : reader.GetMethodDefinition(method).GetGenericParameters().Select(handle => reader.GetString(reader.GetGenericParameter(handle).Name)).ToArray());
    private string Generic(GenericParameterHandleCollection parameters) => parameters.Count == 0 ? "" :
        "<" + string.Join(", ", parameters.Select(handle => reader.GetString(reader.GetGenericParameter(handle).Name))) + ">";
    internal string Type()
    {
        var definition = reader.GetTypeDefinition(owner);
        var context = Names(); var bases = new List<string>();
        if (!definition.BaseType.IsNil) bases.Add(Type(definition.BaseType, context).Display);
        var kind = definition.Attributes.HasFlag(TypeAttributes.Interface) ? "interface" : bases.FirstOrDefault() switch
        { "System.Enum" => "enum", "System.ValueType" => "struct", "System.MulticastDelegate" => "delegate", _ => "class" };
        bases.RemoveAll(name => name is "System.Object" or "System.Enum" or "System.ValueType" or "System.MulticastDelegate");
        bases.AddRange(definition.GetInterfaceImplementations().Select(handle => Type(reader.GetInterfaceImplementation(handle).Interface, context).Display));
        var arguments = context.Type.Select(name => new MetadataType(name, name)).ToImmutableArray();
        var display = arguments.Length == 0 ? OwnerName : types.GetGenericInstantiation(new(OwnerName, OwnerName), arguments).Display;
        var modifiers = kind == "class" ? definition.Attributes.HasFlag(TypeAttributes.Abstract) && definition.Attributes.HasFlag(TypeAttributes.Sealed) ? "static " :
            definition.Attributes.HasFlag(TypeAttributes.Abstract) ? "abstract " : definition.Attributes.HasFlag(TypeAttributes.Sealed) ? "sealed " : "" : "";
        return "public " + modifiers + kind + " " + display + (bases.Count == 0 ? "" : " : " + string.Join(", ", bases)) + Constraints(definition.GetGenericParameters(), context);
    }
    internal (string Signature, string Id) Method(MethodDefinitionHandle handle)
    {
        var definition = reader.GetMethodDefinition(handle); var name = reader.GetString(definition.Name);
        var context = Names(handle); var signature = definition.DecodeSignature(types, context);
        var parameters = definition.GetParameters().Select(reader.GetParameter).ToDictionary(parameter => parameter.SequenceNumber);
        var arguments = signature.ParameterTypes.Select((type, index) =>
        {
            var hasParameter = parameters.TryGetValue(index + 1, out var parameter);
            var byref = type.Xml.EndsWith('@');
            var mode = byref ? parameter.Attributes.HasFlag(ParameterAttributes.Out) ? "out " :
                parameter.Attributes.HasFlag(ParameterAttributes.In) ? "in " : "ref " : hasParameter && HasAttribute(parameter.GetCustomAttributes(), "System.ParamArrayAttribute") ? "params " : "";
            var display = byref ? type.Display.Replace("&", "", StringComparison.Ordinal) : type.Display;
            var parameterName = parameter.Name.IsNil ? "arg" + index : reader.GetString(parameter.Name);
            return mode + display + " " + parameterName + (hasParameter && !parameter.GetDefaultValue().IsNil ? " = " + Constant(parameter.GetDefaultValue()) :
                parameter.Attributes.HasFlag(ParameterAttributes.Optional) ? " [optional]" : "");
        });
        var modifier = definition.Attributes.HasFlag(MethodAttributes.Static) ? "static " : "";
        var displayName = name == ".ctor" ? MetadataSignatureTypes.WithoutArity(OwnerName) : OwnerName + "." + name;
        var display = "public " + modifier + (name == ".ctor" ? "" : signature.ReturnType.Display + " ") + displayName +
            Generic(definition.GetGenericParameters()) + "(" + string.Join(", ", arguments) + ")" + Constraints(definition.GetGenericParameters(), context);
        var id = "M:" + OwnerName + "." + name.Replace('.', '#') + (context.Method.Length == 0 ? "" : "``" + context.Method.Length) + XmlParameters(signature.ParameterTypes);
        if (name is "op_Implicit" or "op_Explicit" or "op_CheckedExplicit" or "op_CheckedImplicit") id += "~" + signature.ReturnType.Xml;
        return (display, id);
    }
    internal (string Signature, string Id) Property(PropertyDefinitionHandle handle)
    {
        var definition = reader.GetPropertyDefinition(handle); var name = reader.GetString(definition.Name);
        var signature = definition.DecodeSignature(types, Names()); var accessors = definition.GetAccessors();
        var getter = Public(accessors.Getter); var setter = Public(accessors.Setter);
        var method = reader.GetMethodDefinition(getter ? accessors.Getter : accessors.Setter);
        return ("public " + (method.Attributes.HasFlag(MethodAttributes.Static) ? "static " : "") + signature.ReturnType.Display + " " + OwnerName + "." + name +
            (signature.ParameterTypes.Length == 0 ? "" : "[" + string.Join(", ", signature.ParameterTypes.Select(type => type.Display)) + "]") +
            " { " + (getter ? "get; " : "") + (setter ? "set; " : "") + "}", "P:" + OwnerName + "." + name + XmlParameters(signature.ParameterTypes));
    }
    internal (string Signature, string Id) Field(FieldDefinitionHandle handle)
    {
        var definition = reader.GetFieldDefinition(handle); var name = reader.GetString(definition.Name);
        return ("public " + (definition.Attributes.HasFlag(FieldAttributes.Literal) ? "const " : definition.Attributes.HasFlag(FieldAttributes.Static) ? "static " : "") +
            (definition.Attributes.HasFlag(FieldAttributes.InitOnly) ? "readonly " : "") + definition.DecodeSignature(types, Names()).Display + " " + OwnerName + "." + name +
            (definition.GetDefaultValue().IsNil ? "" : " = " + Constant(definition.GetDefaultValue())), "F:" + OwnerName + "." + name);
    }
    internal (string Signature, string Id) Event(EventDefinitionHandle handle)
    {
        var definition = reader.GetEventDefinition(handle); var name = reader.GetString(definition.Name); var accessors = definition.GetAccessors();
        var method = reader.GetMethodDefinition(Public(accessors.Adder) ? accessors.Adder : accessors.Remover);
        return ("public " + (method.Attributes.HasFlag(MethodAttributes.Static) ? "static " : "") + "event " + Type(definition.Type, Names()).Display + " " + OwnerName + "." + name,
            "E:" + OwnerName + "." + name);
    }
    internal bool Public(MethodDefinitionHandle handle) => !handle.IsNil && (reader.GetMethodDefinition(handle).Attributes & MethodAttributes.MemberAccessMask) == MethodAttributes.Public;
    private bool HasAttribute(CustomAttributeHandleCollection attributes, string name) => attributes.Any(handle =>
    {
        var attribute = reader.GetCustomAttribute(handle);
        var type = attribute.Constructor.Kind == HandleKind.MemberReference ? reader.GetMemberReference((MemberReferenceHandle)attribute.Constructor).Parent :
            reader.GetMethodDefinition((MethodDefinitionHandle)attribute.Constructor).GetDeclaringType();
        return Type(type, Names()).Xml == name;
    });
    private MetadataType Type(EntityHandle handle, GenericNames context) => handle.Kind switch
    {
        HandleKind.TypeDefinition => types.GetTypeFromDefinition(reader, (TypeDefinitionHandle)handle, 0),
        HandleKind.TypeReference => types.GetTypeFromReference(reader, (TypeReferenceHandle)handle, 0),
        HandleKind.TypeSpecification => types.GetTypeFromSpecification(reader, context, (TypeSpecificationHandle)handle, 0),
        _ => throw new BadImageFormatException("Invalid metadata type handle.")
    };
    private string Constraints(GenericParameterHandleCollection parameters, GenericNames context)
    {
        var parts = new List<string>();
        foreach (var handle in parameters)
        {
            var parameter = reader.GetGenericParameter(handle); var constraints = new List<string>();
            if (parameter.Attributes.HasFlag(GenericParameterAttributes.ReferenceTypeConstraint)) constraints.Add("class");
            if (parameter.Attributes.HasFlag(GenericParameterAttributes.NotNullableValueTypeConstraint)) constraints.Add("struct");
            constraints.AddRange(parameter.GetConstraints().Select(item => Type(reader.GetGenericParameterConstraint(item).Type, context).Display));
            if (parameter.Attributes.HasFlag(GenericParameterAttributes.DefaultConstructorConstraint)) constraints.Add("new()");
            if (constraints.Count != 0) parts.Add(" where " + reader.GetString(parameter.Name) + ": " + string.Join(", ", constraints));
        }
        return string.Concat(parts);
    }
    private static string XmlParameters(IEnumerable<MetadataType> parameters)
    { var values = parameters.Select(type => type.Xml).ToArray(); return values.Length == 0 ? "" : "(" + string.Join(',', values) + ")"; }
    private string Constant(ConstantHandle handle)
    {
        var constant = reader.GetConstant(handle); var blob = reader.GetBlobReader(constant.Value);
        return constant.TypeCode switch
        {
            ConstantTypeCode.NullReference => "null", ConstantTypeCode.Boolean => blob.ReadBoolean() ? "true" : "false",
            ConstantTypeCode.Char => "'" + (char)blob.ReadUInt16() + "'", ConstantTypeCode.String => "\"" + blob.ReadUTF16(blob.Length).Replace("\"", "\\\"", StringComparison.Ordinal) + "\"",
            ConstantTypeCode.SByte => blob.ReadSByte().ToString(CultureInfo.InvariantCulture), ConstantTypeCode.Byte => blob.ReadByte().ToString(CultureInfo.InvariantCulture),
            ConstantTypeCode.Int16 => blob.ReadInt16().ToString(CultureInfo.InvariantCulture), ConstantTypeCode.UInt16 => blob.ReadUInt16().ToString(CultureInfo.InvariantCulture),
            ConstantTypeCode.Int32 => blob.ReadInt32().ToString(CultureInfo.InvariantCulture), ConstantTypeCode.UInt32 => blob.ReadUInt32().ToString(CultureInfo.InvariantCulture),
            ConstantTypeCode.Int64 => blob.ReadInt64().ToString(CultureInfo.InvariantCulture), ConstantTypeCode.UInt64 => blob.ReadUInt64().ToString(CultureInfo.InvariantCulture),
            ConstantTypeCode.Single => blob.ReadSingle().ToString(CultureInfo.InvariantCulture), ConstantTypeCode.Double => blob.ReadDouble().ToString(CultureInfo.InvariantCulture),
            _ => throw new BadImageFormatException("Invalid metadata constant.")
        };
    }
}
