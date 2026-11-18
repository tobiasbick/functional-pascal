using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Syntax;
using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;

namespace Fpas.Compiler.Analysis;

/// <summary>FPAS usage for one selected metadata member.</summary>
/// <param name="Status">template for a checked unit with caller inputs, example for a checked unit without inputs, or unavailable.</param>
/// <param name="Source">Complete checked FPAS unit, or null when the member cannot be demonstrated.</param>
/// <param name="Inputs">Wrapper parameter names to supply at the call site, in declaration order.</param>
/// <param name="Reason">Explanation of the result or a specific limitation.</param>
public sealed record ApiUsage(string Status, string? Source, IReadOnlyList<string> Inputs, string Reason)
{
    internal static ApiUsage Unavailable(string reason) => new("unavailable", null, [], reason);
}

/// <summary>Builds usage units from the compiler's actual imported signatures and verifies the selected member binding.</summary>
public static class ApiUsageTemplates
{
    /// <summary>Checks one generated unit against the project's reference set without executing it.</summary>
    /// <param name="project">Evaluated references, captured images and diagnostic policy.</param>
    /// <param name="owner">Metadata declaring type name, including arity.</param>
    /// <param name="moduleId">Module version ID associated with the selected member token.</param>
    /// <param name="memberToken">Method, property or field definition token in that module.</param>
    /// <param name="typeArguments">Concrete FPAS types for enclosing types, then the method.</param>
    /// <param name="write">Generate an assignment for a property, indexer or field.</param>
    /// <param name="token">Cancels metadata inspection and binding.</param>
    /// <returns>A checked usage unit or an explanation.</returns>
    public static ApiUsage Create(AnalysisRequest project, string owner, Guid moduleId, int memberToken,
        IReadOnlyList<string> typeArguments, bool write, CancellationToken token = default)
    {
        var source = new SourceText("ApiUsage.fpas", "", token);
        try
        {
            using var catalog = ReferenceCatalog.Open(project.References, source, project.ReferenceImages, captureFingerprints: true);
            var type = catalog.FindType(owner, 0, source);
            var reader = type.Assembly.Reader;
            var handle = MetadataTokens.EntityHandle(memberToken);
            bool belongs = handle.Kind switch
            {
                HandleKind.MethodDefinition => type.Definition.GetMethods().Contains((MethodDefinitionHandle)handle),
                HandleKind.PropertyDefinition => type.Definition.GetProperties().Contains((PropertyDefinitionHandle)handle),
                HandleKind.FieldDefinition => type.Definition.GetFields().Contains((FieldDefinitionHandle)handle),
                _ => false
            };
            if (!belongs || reader.GetGuid(reader.GetModuleDefinition().Mvid) != moduleId)
                return ApiUsage.Unavailable("The selected definition does not belong to this reference type.");

            int ownerArity = type.Definition.GetGenericParameters().Count;
            int methodArity = handle.Kind == HandleKind.MethodDefinition ? reader.GetMethodDefinition((MethodDefinitionHandle)handle).GetGenericParameters().Count : 0;
            if (typeArguments.Count != ownerArity + methodArity)
                return ApiUsage.Unavailable($"Expected concrete type arguments for the declaring type ({ownerArity}) and method ({methodArity}), in that order. Received {typeArguments.Count}.");
            var arguments = typeArguments.Select(arg => UsageTypeArguments.Resolve(arg, catalog, token)).ToArray();
            var ownerType = ownerArity == 0 ? catalog.Symbol(type) : catalog.Construct(type, arguments[..ownerArity], source, 0);

            if (handle.Kind == HandleKind.FieldDefinition)
            {
                var usage = FieldUsage(catalog, type, ownerType, (FieldDefinitionHandle)handle, write, source);
                return Verify(project, usage, type, handle, write ? "write" : "read", token);
            }

            string? propertyName = null;
            MethodDefinitionHandle methodHandle;
            if (handle.Kind == HandleKind.PropertyDefinition)
            {
                var property = reader.GetPropertyDefinition((PropertyDefinitionHandle)handle);
                propertyName = reader.GetString(property.Name);
                var accessors = property.GetAccessors();
                methodHandle = write ? accessors.Setter : accessors.Getter;
                if (methodHandle.IsNil || (reader.GetMethodDefinition(methodHandle).Attributes & MethodAttributes.MemberAccessMask) != MethodAttributes.Public)
                    return ApiUsage.Unavailable($"The selected property has no public {(write ? "setter" : "getter")}.");
            }
            else
            {
                if (write) return ApiUsage.Unavailable("A method cannot be assigned.");
                methodHandle = (MethodDefinitionHandle)handle;
            }

            var definitionMethod = reader.GetMethodDefinition(methodHandle);
            if ((definitionMethod.Attributes & MethodAttributes.MemberAccessMask) != MethodAttributes.Public)
                return ApiUsage.Unavailable("Usage generation requires a public member.");
            var methodName = reader.GetString(definitionMethod.Name);
            bool constructor = methodName == ".ctor";
            if (constructor && type.Definition.Attributes.HasFlag(TypeAttributes.Abstract))
                return ApiUsage.Unavailable("An abstract type cannot be constructed.");
            if (propertyName is null && !constructor && definitionMethod.Attributes.HasFlag(MethodAttributes.SpecialName))
                return ApiUsage.Unavailable("Operators and accessors require a different FPAS expression.");
            if (definitionMethod.Attributes.HasFlag(MethodAttributes.Static | MethodAttributes.Virtual))
                return ApiUsage.Unavailable("A static virtual interface member needs a constrained type parameter.");
            catalog.CheckMethodSignature(type.Assembly, definitionMethod, owner + "." + methodName, source, 0, null, propertyName is null);
            var method = catalog.ReadMethod(ownerType, methodHandle,
                methodArity == 0 ? null : new TypeList(arguments[ownerArity..]), allowSpans: propertyName is null);
            if (method is null) return ApiUsage.Unavailable("The compiler cannot represent this CLR signature.");
            if (constructor && catalog.RecordInfo(ownerType) is { } record && record.Constructor.DefinitionHandle == methodHandle)
                method = record.Constructor;
            if (!catalog.ValidMethodArguments(method))
                return ApiUsage.Unavailable("The selected method's generic constraints reject the supplied type arguments.");
            if (propertyName is null)
            {
                ApiUsage CheckedCall(bool namedArguments) => Verify(project, UsageSource.Call(catalog, type, method, namedArguments),
                    type, methodHandle, constructor ? "construct" : "invoke", token);
                // Overloads can use the same names in different positions. Verify the named form before offering it.
                if (NamedCallArguments.Useful(method) && CheckedCall(true) is { Source: not null } named) return named;
                return CheckedCall(false);
            }

            if (method.ParameterInfo?.Any(parameter => parameter.ByRef) == true)
                return ApiUsage.Unavailable("FPAS property and indexer access requires by-value parameters.");
            if (write && (method.Parameters.Count == 0 || method.Result != TypeSymbol.Void) || !write && method.Result == TypeSymbol.Void)
                return ApiUsage.Unavailable("The selected accessor has an invalid property signature.");
            var valueType = write ? method.Parameters[^1] : method.Result;
            if (!write && method.Parameters.Count == 0 && catalog.Property(ownerType, propertyName, method.IsStatic) is { } imported)
                valueType = imported.Type;
            return Verify(project, UsageSource.Property(type, propertyName, method, write, valueType), type, methodHandle, write ? "write" : "read", token);
        }
        catch (CompilationFailure failure)
        {
            return ApiUsage.Unavailable(failure.Diagnostic.Code + ": " + failure.Diagnostic.Message);
        }
        catch (ArgumentException error)
        {
            return ApiUsage.Unavailable("The selected signature cannot be spelled in an FPAS usage unit: " + error.Message);
        }
    }

    private static ApiUsage FieldUsage(ReferenceCatalog catalog, ImportedType owner, TypeSymbol ownerType,
        FieldDefinitionHandle handle, bool write, SourceText source)
    {
        var reader = owner.Assembly.Reader;
        var definition = reader.GetFieldDefinition(handle);
        if ((definition.Attributes & FieldAttributes.FieldAccessMask) != FieldAttributes.Public)
            return ApiUsage.Unavailable("Usage generation requires a public field.");
        var name = reader.GetString(definition.Name);
        catalog.CheckFieldSignature(owner.Assembly, definition, owner.FullName + "." + name, source, 0);
        var field = catalog.Field(ownerType, name, definition.Attributes.HasFlag(FieldAttributes.Static));
        if (field is null || field.DefinitionHandle != handle || field.Owner != ownerType)
            return ApiUsage.Unavailable("The compiler cannot represent the selected field.");
        if (write && field.IsReadOnly) return ApiUsage.Unavailable("A read-only or constant field cannot be assigned.");
        return UsageSource.Field(owner, field, write);
    }

    private static ApiUsage Verify(AnalysisRequest project, ApiUsage usage, ImportedType owner, EntityHandle selected, string role, CancellationToken token)
    {
        if (usage.Source is null) return usage;
        var result = FpasCompiler.Analyze(project with
        {
            Sources = [new CompilationSource("ApiUsage.fpas", usage.Source)], AssemblyName = "ApiUsage",
            Target = CompilationTarget.Library, ValidateDocumentation = false
        }, token);
        if (!result.Success)
        {
            var error = result.Diagnostics.FirstOrDefault(d => d.Severity == DiagnosticSeverity.Error);
            return ApiUsage.Unavailable(error is null ? "Compiler verification was incomplete." : error.Code + ": " + error.Message);
        }
        var snapshot = result.Snapshot!;
        var hash = owner.Assembly.ContentHash;
        var expected = MetadataTokens.GetToken(selected);
        // Named storage is indexed as a symbol occurrence. Indexers are indexed by their bound accessor.
        if (role is "read" or "write" && snapshot.Occurrences.Any(site => site.Role == role && site.AccessToken == expected &&
            snapshot.Symbol(site.Symbol).Metadata?.AssemblyHash == hash)) return usage;
        var operation = role switch { "read" => "property_get", "write" => "property_set", _ => role };
        bool exact = snapshot.Operations.Any(op => (op.Kind == operation || operation == "invoke" && op.Kind == "delegate_invoke") &&
            op.Operation is { } bound && bound.DefinitionToken == expected && bound.AssemblyHash == hash);
        return exact ? usage : ApiUsage.Unavailable("The generated expression did not bind to the selected member or overload.");
    }
}
