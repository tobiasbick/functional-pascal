using System.Reflection;
using System.Reflection.Metadata;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding;

internal sealed partial class ReferenceCatalog
{
    internal ImportedMethod FindMethod(TypeSymbol owner, string name, IReadOnlyList<TypeSymbol> arguments, int position, SourceText source,
        bool isStatic = true, bool optional = false, TypeList? methodArguments = null,
        Func<int, TypeSymbol, bool>? accepts = null, IReadOnlyList<bool>? delegateSources = null, RecordSymbol? access = null, IReadOnlyList<ImportedMethod>? candidates = null)
    {
        var methods = (candidates ?? Methods(owner, name, isStatic, methodArguments, access)).Where(method => method.Parameters.Count == arguments.Count &&
            !method.Parameters.Where((parameter, index) => !(accepts?.Invoke(index, parameter) ?? Applicable(arguments[index], parameter))).Any()).ToArray();
        var matches = methods.Where(ValidMethodArguments).ToArray();
        var best = matches.Where(candidate => !matches.Any(other => Better(other, candidate, arguments, delegateSources))).ToArray();
        if (best.Length == 1) return best[0];
        if (optional && best.Length == 0) return null!;
        var code = best.Length == 0 && methods.Length > 0 ? "FP3018" : "FP3003";
        throw source.Error(code, $"{(best.Length == 0 ? "No supported public overload" : "Ambiguous overloads")} for '{owner}.{name}'.", position);
    }

    internal IReadOnlyList<ImportedMethod> Methods(TypeSymbol owner, string name, bool isStatic, TypeList? methodArguments = null, RecordSymbol? access = null, bool includeOpen = false)
    {
        var result = new List<ImportedMethod>();
        var signatures = new HashSet<string>(StringComparer.Ordinal);
        foreach (var type in name == ".ctor" ? new[] { owner } : Hierarchy(owner))
        {
            if (type.Kind == TypeKind.Scalar && !type.NullableValue)
            {
                foreach (var method in ScalarMethods(type).Where(method => method.Name == name && method.IsStatic == isStatic && methodArguments is null))
                    if (signatures.Add("0:" + string.Join(';', method.Parameters.Select(parameter => parameter.Erased)))) result.Add(method);
                continue;
            }
            if (type.Union is { } union)
            {
                if (methodArguments is null)
                    foreach (var method in union.Methods.Where(method => method.IsPublic && method.Name == name && !method.IsInstance == isStatic))
                    {
                        var imported = InstantiateSource(method, type, null);
                        if (signatures.Add("0:" + string.Join(';', imported.DelegateParameters.Select(parameter => parameter.Erased)))) result.Add(imported);
                    }
                continue;
            }
            if (type.Record is { } record)
            {
                foreach (var method in record.Methods.Where(method => (includeOpen || method.GenericParameters.Count == (methodArguments?.Count ?? 0)) && method.Name == name && !method.IsInstance == isStatic && (method.IsPublic || record == access)))
                    {
                        var arguments = methodArguments ?? (method.GenericParameters.Count == 0 ? null : new TypeList(method.GenericParameters.Select(parameter => parameter.Type)));
                        var imported = InstantiateSource(method, type, arguments);
                        var key = method.GenericParameters.Count + ":" + string.Join(';', imported.DelegateParameters.Select(parameter => parameter.Erased));
                        if (signatures.Add(key)) result.Add(imported);
                    }
                continue;
            }
            var definition = Definition(type);
            foreach (var handle in definition.Definition.GetMethods())
            {
                var method = definition.Assembly.Reader.GetMethodDefinition(handle);
                if (definition.Assembly.Reader.GetString(method.Name) != name ||
                    method.Attributes.HasFlag(MethodAttributes.Static) != isStatic ||
                    (method.Attributes & MethodAttributes.MemberAccessMask) != MethodAttributes.Public || !includeOpen && method.GetGenericParameters().Count != (methodArguments?.Count ?? 0)) continue;
                var imported = ConstrainStatic(ReadMethod(type, handle, methodArguments, allowOpen: includeOpen), owner);
                if (imported is null) continue;
                var key = method.GetGenericParameters().Count + ":" + string.Join(';', imported.DelegateParameters.Select(parameter => parameter.Erased));
                if (imported.ConstrainedType is not null || signatures.Add(key)) result.Add(imported);
            }
        }
        return MostSpecificStatic(result);
    }

    private ImportedMethod? ReadMethod(TypeSymbol owner, MethodDefinitionHandle handle, TypeList? methodArguments = null, bool includePrivate = false, bool allowOpen = false)
    {
        var definition = Definition(owner);
        var method = definition.Assembly.Reader.GetMethodDefinition(handle);
        var raw = method.DecodeSignature(new SignatureTypes(this, definition.Assembly, includePrivate), null);
        var keep = allowOpen && methodArguments is null;
        var result = Substitute(NullabilityMetadata.Parameter(raw.ReturnType, definition, method, 0), owner, methodArguments, keep);
        var parameters = raw.ParameterTypes.Select((type, index) => Substitute(NullabilityMetadata.Parameter(type.Kind == TypeKind.ByRef ? type.Element! : type, definition, method, index + 1), owner, methodArguments, keep)).ToArray();
        bool Supported(TypeSymbol type) => allowOpen ? UsableTemplate(type) : Usable(type);
        if (raw.Header.CallingConvention != SignatureCallingConvention.Default || raw.ReturnType.Kind == TypeKind.ByRef || !Supported(result) ||
            parameters.Any(type => !Supported(type) || type == TypeSymbol.Void)) return null;
        if (keep && method.GetGenericParameters().Count != 0)
            methodArguments = new(Enumerable.Range(0, method.GetGenericParameters().Count).Select(index => new TypeSymbol(TypeKind.MethodParameter, ParameterIndex: index)));
        return new(owner, definition.Assembly.Reader.GetString(method.Name), result, parameters,
            method.Attributes.HasFlag(MethodAttributes.Static), method.Attributes.HasFlag(MethodAttributes.Virtual), raw.ReturnType, raw.ParameterTypes,
            MethodArguments: methodArguments, DefinitionHandle: handle, ParameterInfo: ReadParameters(definition, method, parameters, raw.ParameterTypes));
    }

    internal bool Better(ImportedMethod left, ImportedMethod right, IReadOnlyList<TypeSymbol> arguments, IReadOnlyList<bool>? delegateSources = null, bool tieBreak = true)
    {
        var better = false;
        for (var index = 0; index < arguments.Count; index++)
        {
            var l = left.Parameters[index]; var r = right.Parameters[index]; var arg = arguments[index];
            if (l.IsReference) l = l.NonNullable;
            if (r.IsReference) r = r.NonNullable;
            if (arg.IsReference) arg = arg.NonNullable;
            if (l == r) continue;
            if (delegateSources?[index] != true)
            {
                if (r == arg) return false;
                if (l == arg) { better = true; continue; }
            }
            if (!CanAssign(l, r) && !(NumericFacts.Integral(l) && NumericFacts.Integral(r) && !NumericFacts.Unsigned(l) && NumericFacts.Unsigned(r) && NumericFacts.Bits(l) <= NumericFacts.Bits(r))) return false;
            better = true;
        }
        if (better) return true;
        if (!tieBreak) return false;
        if (!left.Parameters.Select(parameter => parameter.Erased).SequenceEqual(right.Parameters.Select(parameter => parameter.Erased))) return false;
        if (left.MethodArguments is null && right.MethodArguments is { Count: > 0 }) return true;
        return left.MethodArguments is { Count: > 0 } && right.MethodArguments is { Count: > 0 } && MoreSpecific(left, right);
    }

    internal ImportedProperty? Property(TypeSymbol owner, string name, bool isStatic, RecordSymbol? access = null, SourceText? source = null, int position = 0)
    {
        var constrained = new List<(TypeSymbol Owner, ImportedProperty Property)>();
        foreach (var type in Hierarchy(owner))
        {
            if (type.Kind == TypeKind.Scalar)
            {
                var info = ScalarInfo(type)!;
                if (!isStatic && name == "Value" && info.Category != ScalarCategory.Enum) return new(info.Base, DomainGetter(info), null);
                continue;
            }
            if (type.Record is { } record)
            {
                var field = record.Fields.SingleOrDefault(field => field.Name == name);
                if (!isStatic && field is not null && (field.Syntax.IsPublic || record == access))
                {
                    var getter = InstantiateSource(field.Getter, type, null);
                    return new(getter.Result, getter, null);
                }
                continue;
            }
            if (type.Union is { } union)
            {
                if (!isStatic && name == "Tag") return new(TypeSymbol.Int32, InstantiateSource(union.Method(GeneratedMethodKind.Tag), type, null), null);
                var member = SourceUnionInfo(type).Cases.FirstOrDefault(member => member.Name == name && member.Fields.Count == 0);
                if (isStatic && member is not null) return new(type, member.Factory, null);
                continue;
            }
            var definition = Definition(type);
            foreach (var handle in definition.Definition.GetProperties())
            {
                var property = definition.Assembly.Reader.GetPropertyDefinition(handle);
                if (definition.Assembly.Reader.GetString(property.Name) != name) continue;
                var signature = property.DecodeSignature(new SignatureTypes(this, definition.Assembly), null);
                if (signature.ParameterTypes.Length != 0 || signature.ReturnType == TypeSymbol.Unsupported) continue;
                var accessors = property.GetAccessors();
                ImportedMethod? Accessor(MethodDefinitionHandle methodHandle)
                {
                    if (methodHandle.IsNil) return null;
                    var method = definition.Assembly.Reader.GetMethodDefinition(methodHandle);
                    return method.Attributes.HasFlag(MethodAttributes.Static) == isStatic &&
                        (method.Attributes & MethodAttributes.MemberAccessMask) == MethodAttributes.Public ? ConstrainStatic(ReadMethod(type, methodHandle), owner) : null;
                }
                var getter = Accessor(accessors.Getter); var setter = Accessor(accessors.Setter);
                if (getter is null && setter is null) continue;
                var result = Substitute(NullabilityMetadata.Member(signature.ReturnType, definition, property.GetCustomAttributes()), type);
                if (getter is not null && !NullabilityMetadata.HasNullable(definition.Assembly.Reader, property.GetCustomAttributes())) result = getter.Result;
                var imported = new ImportedProperty(result, getter is null ? null : getter with { Result = result }, setter);
                if (!owner.IsParameter || !isStatic) return imported;
                constrained.Add((type, imported));
            }
        }
        var visible = constrained.Where(candidate => !constrained.Any(other => other.Owner != candidate.Owner && Derives(other.Owner, candidate.Owner))).ToArray();
        if (visible.Length > 1) throw (source ?? new SourceText("<constraints>", "")).Error("FP3003", $"Ambiguous static interface property '{owner}.{name}'.", position);
        return visible.SingleOrDefault().Property;
    }

    internal ImportedField? Field(TypeSymbol owner, string name, bool isStatic)
    {
        foreach (var type in Hierarchy(owner))
        {
            if (type.Scalar is { } scalar)
            {
                if (!isStatic) continue;
                if (scalar.Syntax.Category == ScalarCategory.Enum && scalar.Members.FirstOrDefault(member => member.Name.Text == name) is { } member)
                    return new(type, name, type, true, true, member.Value, true);
                if (scalar.Syntax.Category == ScalarCategory.Range && name is "LowerBound" or "UpperBound")
                {
                    var info = ScalarInfo(type)!;
                    return new(type, name, info.Base, true, true, NumericValue.ConvertTo(name == "LowerBound" ? info.Lower!.Value : info.Upper!.Value, info.Base, true), true);
                }
                continue;
            }
            if (type.Record is not null || type.Union is not null) continue;
            var definition = Definition(type);
            foreach (var handle in definition.Definition.GetFields())
            {
                var field = definition.Assembly.Reader.GetFieldDefinition(handle);
                if (definition.Assembly.Reader.GetString(field.Name) != name || field.Attributes.HasFlag(FieldAttributes.Static) != isStatic ||
                    (field.Attributes & FieldAttributes.FieldAccessMask) != FieldAttributes.Public) continue;
                var result = field.DecodeSignature(new SignatureTypes(this, definition.Assembly), null);
                if (result == TypeSymbol.Unsupported) continue;
                var raw = result;
                result = Substitute(NullabilityMetadata.Member(result, definition, field.GetCustomAttributes()), type);
                if (!Usable(result)) continue;
                var literal = field.Attributes.HasFlag(FieldAttributes.Literal);
                object? value = null;
                if (literal)
                    value = MetadataConstants.Read(definition.Assembly.Reader, field.GetDefaultValue());
                else if (isStatic && result == TypeSymbol.Decimal && MetadataConstants.Decimal(definition.Assembly.Reader, field.GetCustomAttributes()) is { } decimalValue)
                { literal = true; value = decimalValue; }
                return new(type, name, result, isStatic, literal || field.Attributes.HasFlag(FieldAttributes.InitOnly), value, literal, raw);
            }
        }
        return null;
    }
}
