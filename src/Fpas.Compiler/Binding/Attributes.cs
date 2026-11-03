namespace Fpas.Compiler.Binding;

internal sealed record AttributeValue(TypeSymbol Type, object? Value);
internal sealed record NamedAttributeValue(string Name, TypeSymbol Type, AttributeValue Value, bool IsField);
internal sealed record BoundAttribute(ImportedMethod Constructor, IReadOnlyList<AttributeValue> Arguments, IReadOnlyList<NamedAttributeValue> Named);
