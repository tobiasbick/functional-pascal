namespace Fpas.Compiler.Binding;

internal sealed partial class ReferenceCatalog
{
    internal TypeSymbol CaptureStorage(CaptureSymbol capture) => capture.Mutable ? CellType(capture.Type) : capture.Type;

    internal ImportedField CaptureField(CaptureSymbol capture)
        => new(capture.Owner.Type, "<Capture" + capture.Owner.Captures.IndexOf(capture) + ">", CaptureStorage(capture), false, true,
            SignatureType: ClosureSignature(CaptureStorage(capture), capture.Owner));

    internal ImportedMethod ClosureMethod(ClosureSymbol closure, bool constructor)
    {
        var parameters = constructor ? closure.Captures.Select(CaptureStorage).ToArray() : closure.Body.Symbol.Parameters.Select(parameter => parameter.Type).ToArray();
        var result = constructor ? TypeSymbol.Void : closure.Body.Symbol.Result;
        return new(closure.Type, constructor ? ".ctor" : "<Invoke>", result, parameters, false,
            SignatureResult: ClosureSignature(result, closure), SignatureParameters: (constructor ? parameters : closure.Body.Symbol.Parameters.Select(parameter => parameter.SignatureType)).Select(parameter => ClosureSignature(parameter, closure)).ToArray(),
            ParameterInfo: constructor ? null : closure.Body.Symbol.Parameters);
    }

    private TypeSymbol ClosureSignature(TypeSymbol type, ClosureSymbol closure)
    {
        if (type.IsParameter)
        {
            var index = closure.GenericParameters.ToList().IndexOf(type.Parameter!);
            if (index < 0) throw new InvalidOperationException("A closure contains an unrelated type parameter.");
            var variable = new TypeSymbol(TypeKind.TypeParameter, ParameterIndex: index, IsNullable: type.IsNullable && !type.NullableValue);
            return type.NullableValue ? Symbol(Required("System.Nullable`1")) with { Arguments = new([variable]) } : variable;
        }
        return type with
        {
            Element = type.Element is null ? null : ClosureSignature(type.Element, closure),
            Arguments = type.Arguments is null ? null : new(type.Arguments.Select(argument => ClosureSignature(argument, closure)))
        };
    }
}
