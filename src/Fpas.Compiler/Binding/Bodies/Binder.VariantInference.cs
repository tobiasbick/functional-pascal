using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Bodies;

internal sealed partial class Binder
{
    private BoundExpression InferVariant(OpenVariantInfo open, string name, IReadOnlyList<ExpressionSyntax>? arguments, TypeSymbol? expected, bool check, int position)
    {
        var member = open.Info.Cases.FirstOrDefault(item => item.Name == name)
            ?? throw source.Error("FP3023", "Unknown case on a generic variant owner.", position);
        if (arguments is null ? member.Fields.Count != 0 : member.Fields.Count == 0)
            throw source.Error("FP3023", "Payload cases require arguments; fieldless cases have no parentheses.", position);
        var inference = new TypeInference(references, open.Variables);
        if (expected is not null && SameVariantOwner(open.Info.Type, expected) && expected.TypeArguments.Count == open.Variables.Count)
            inference.Add(open.Info.Type, expected.NonNullable, InferenceBound.Exact);
        var supplied = DataArguments(arguments ?? [], member.Fields.Select(field => field.Name).ToArray(), position);
        var syntax = supplied.Select(argument => argument.Value).ToArray();
        var parameters = supplied.Select(argument => member.Fields[argument.Slot].Type).ToArray();
        var bound = BindInferenceValues(syntax, check, out var argumentFacts, parameters, inference);
        for (var index = 0; index < bound.Length; index++) if (bound[index] is { } value) inference.Add(parameters[index], value);
        var inferred = inference.Fix() ?? throw source.Error("FP3026", "Cannot infer a unique variant instantiation from its payloads and expected type; specify the owner type arguments.", position);
        var owner = references.CloseVariant(open, inferred, source, position);
        var closed = references.VariantInfo(owner)!.Cases.Single(item => item.Name == name);
        if (arguments is null)
        {
            ForgetCapturedFacts();
            return new BoundCall(closed.Factory, []);
        }
        var values = BindDeferredArguments(syntax, bound, supplied.Select(argument => closed.Fields[argument.Slot].Type).ToArray(), check, argumentFacts);
        return BindVariantFactory(closed, arguments, check, position, values);
    }

    private IReadOnlyList<(int Slot, ExpressionSyntax Value)> DataArguments(IReadOnlyList<ExpressionSyntax> arguments, IReadOnlyList<string> names, int position)
    {
        var result = new List<(int, ExpressionSyntax)>(); var slots = new HashSet<int>(); var positional = 0; var named = false;
        foreach (var argument in arguments)
        {
            var slot = positional++; var expression = argument;
            if (argument is NamedArgumentSyntax supplied)
            { named = true; slot = names.ToList().IndexOf(supplied.Name.Text); expression = supplied.Value; }
            else if (named) throw source.Error("FP3022", "Positional arguments must precede named data-construction arguments.", argument.Position);
            if (slot < 0 || slot >= names.Count || !slots.Add(slot))
                throw source.Error("FP3022", "Unknown, duplicate, or excess data-construction argument.", argument.Position);
            result.Add((slot, expression));
        }
        return result;
    }
}
