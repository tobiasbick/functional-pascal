// [Documentation](../../../../docs/pascal/tools/diagnostics.md#call-diagnostics)
using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Syntax;
using System.Globalization;

namespace Fpas.Compiler.Binding.Bodies;

internal sealed partial class Binder
{
    private const int CallDiagnosticLimit = 8;
    private readonly record struct ShapeRejection(string? Kind, int Index, int Slot = -1);
    private sealed record RejectedCall(CallShape Shape, int FirstMismatch, int Mismatches);

    private CompilationFailure DiagnoseCall(IReadOnlyList<ImportedMethod> templates, IReadOnlyList<CallShape> shapes,
        IReadOnlyList<ExpressionSyntax> syntax, IReadOnlyList<ExpressionSyntax> arguments, IReadOnlyList<BoundExpression?> values,
        bool inferGenerics, int position, Action? noApplicable)
    {
        var mismatches = new List<RejectedCall>();
        var missingTypes = new List<ImportedMethod>();
        var constraints = new List<(ImportedMethod Method, ReferenceCatalog.GenericConstraintFailure Failure)>();
        foreach (var shape in shapes)
        {
            var method = InferCallMethod(shape, values, inferGenerics);
            if (method is null) { missingTypes.Add(shape.Method); continue; }
            if (references.MethodConstraintFailure(method) is { } constraint) { constraints.Add((method, constraint)); continue; }
            var closed = ReferenceEquals(method, shape.Method) ? shape : Shape(method, syntax, shape.Expanded)!;
            var rejected = Enumerable.Range(0, values.Count).Where(index => !AcceptsArgument(values[index], arguments[index], closed, index)).ToArray();
            if (rejected.Length != 0) mismatches.Add(new(closed, rejected[0], rejected.Length));
        }
        if (constraints.Count != 0)
        {
            var failures = constraints.Select(item => item.Failure).Distinct().Take(CallDiagnosticLimit).ToArray();
            var message = failures.Length == 1 ? ConstraintDescription(failures[0]) : "The supplied or inferred type arguments violate generic constraints.";
            return source.Error("FP3018", message, position,
                help: "Check the constraints of the listed signatures and supply compatible type arguments.",
                expectation: new("generic_constraint", failures.Select(failure => failure.Requirement).Distinct().ToArray(),
                    string.Join(" or ", failures.Select(failure => TypeFacts.Display(failure.Argument)).Distinct())),
                related: constraints.Select(item => new DiagnosticNote($"{CallSignature(item.Method)}. {ConstraintDescription(item.Failure)}"))
                    .Distinct().Take(CallDiagnosticLimit).ToArray());
        }
        if (missingTypes.Count != 0 && mismatches.Count == 0)
        {
            var signatures = Signatures(missingTypes);
            return source.Error("FP3026", "Cannot infer a unique set of generic type arguments for this call.", position,
                help: "Supply explicit type arguments with 'of' on the routine name or pass arguments whose types determine them.",
                expectation: new("type_arguments", signatures, "inference failed"),
                related: signatures.Select(signature => new DiagnosticNote($"Type inference failed for {signature}.")).ToArray());
        }
        if (missingTypes.Count == 0) noApplicable?.Invoke();
        if (mismatches.Count != 0) return ArgumentMismatch(mismatches, syntax, arguments, values, position);
        return ShapeMismatch(templates, syntax, position);
    }

    private CompilationFailure ArgumentMismatch(List<RejectedCall> rejected, IReadOnlyList<ExpressionSyntax> syntax,
        IReadOnlyList<ExpressionSyntax> arguments, IReadOnlyList<BoundExpression?> values, int position)
    {
        var minimum = rejected.Min(item => item.Mismatches);
        var closest = rejected.Where(item => item.Mismatches == minimum).ToArray();
        var index = closest[0].FirstMismatch;
        var signatures = Signatures(closest.Select(item => item.Shape.Method));
        var notes = closest.Select(item =>
        {
            var mismatch = item.FirstMismatch;
            return new DiagnosticNote($"{CallSignature(item.Shape.Method)} expects {ParameterDescription(ExpectedParameter(item.Shape, mismatch))} at argument {mismatch + 1}.",
                source.Location(arguments[mismatch].SpanStart, ExpressionLength(arguments[mismatch])));
        }).Distinct().Take(CallDiagnosticLimit).ToArray();
        if (closest.Any(item => item.FirstMismatch != index))
            return source.Error("FP3003", "The closest overloads reject different arguments.", position,
                expectation: new("overload", signatures, "no applicable overload"), related: notes);

        var alternatives = closest.Select(item => ExpectedParameter(item.Shape, index)).Distinct().Take(CallDiagnosticLimit).ToArray();
        var value = values[index];
        var address = value as BoundAddress;
        var actualType = (address?.Target ?? value)?.Type;
        var argument = new DiagnosticArgument(index + 1, (syntax[index] as NamedArgumentSyntax)?.Name.Text,
            actualType is null || actualType == TypeSymbol.Unsupported ? null : TypeFacts.Display(actualType),
            address?.Mode ?? "value", alternatives);
        var expected = alternatives.Select(ParameterDescription).Distinct().ToArray();
        var found = PassingType(argument.Mode, argument.Type ?? "a context-dependent expression");
        var label = argument.Name ?? (arguments[index] is NameExpressionSyntax name ? name.Name.Text : null);
        var display = label is null ? $"Argument {argument.Index}" : $"Argument {argument.Index} ('{label}')";
        return source.Error("FP3003", $"{display} expects {string.Join(" or ", expected)}. Received {found}.",
            arguments[index].SpanStart, ExpressionLength(arguments[index]), expectation: new("argument", expected, found), related: notes, argument: argument);
    }

    private CompilationFailure ShapeMismatch(IReadOnlyList<ImportedMethod> templates, IReadOnlyList<ExpressionSyntax> syntax, int position)
    {
        if (templates.Count == 0) return source.Error("FP3003", "There is no supported signature for this call.", position);
        var rejected = new List<(ImportedMethod Method, ShapeRejection Rejection)>();
        foreach (var method in templates)
        {
            Shape(method, syntax, false, false, out var regular);
            Shape(method, syntax, true, false, out var expanded);
            if (regular.Kind is not null) rejected.Add((method, regular));
            if (expanded.Kind is not null && expanded != regular && expanded.Index > regular.Index) rejected.Add((method, expanded));
        }
        var closest = rejected.GroupBy(item => item.Rejection.Index).OrderByDescending(group => group.Key).FirstOrDefault()?.ToArray() ?? [];
        var signatures = Signatures(closest.Length == 0 ? templates : closest.Select(item => item.Method));
        var first = closest.FirstOrDefault().Rejection;
        if (first.Kind is null || closest.Any(item => item.Rejection.Kind != first.Kind))
            return source.Error("FP3003", "The supplied argument layout does not match a callable signature.", position,
                expectation: new("overload", signatures, "argument layout"), related: CandidateNotes(signatures));

        var at = first.Index < syntax.Count ? syntax[first.Index] : null;
        var named = at as NamedArgumentSyntax;
        var location = named?.Name.Position ?? at?.SpanStart ?? position;
        var length = named?.Name.Length ?? (at is null ? 1 : ExpressionLength(at));
        var supplied = named?.Name.Text ?? (first.Index + 1).ToString(CultureInfo.InvariantCulture);
        var parameters = closest.Where(item => item.Rejection.Slot >= 0)
            .Select(item => item.Method.Parameter(item.Rejection.Slot).Name).Where(name => name.Length != 0).Distinct().ToArray();
        var names = closest.SelectMany(item => item.Method.ParameterInfo ?? [])
            .Select(parameter => parameter.Name).Where(name => name.Length != 0).Distinct().Order(StringComparer.Ordinal).Take(CallDiagnosticLimit).ToArray();
        var (message, expected, found) = first.Kind switch
        {
            "parameter_name" => ($"Unknown parameter name '{supplied}'.", names, supplied),
            "argument_order" => ("Positional arguments must precede named arguments.", new[] { "named argument" }, "positional argument"),
            "duplicate_argument" => ($"Parameter '{supplied}' was already supplied in this call.", parameters, supplied),
            "named_params" => ($"Pass the array as '{supplied}' or supply positional elements.", parameters, supplied),
            "missing_argument" when parameters.Length != 0 =>
                ($"Missing required argument for {string.Join(" or ", parameters.Select(name => $"'{name}'"))}.", parameters, "omitted"),
            _ => ($"The supplied argument count ({syntax.Count}) does not match the closest signatures.",
                closest.Select(item => item.Method.Parameters.Count.ToString(CultureInfo.InvariantCulture)).Distinct().ToArray(),
                syntax.Count.ToString(CultureInfo.InvariantCulture))
        };
        return source.Error("FP3003", message, location, length, expectation: new(first.Kind, expected, found),
            related: CandidateNotes(signatures));
    }

    private CompilationFailure AmbiguousCall(IReadOnlyList<CallShape> best, int position)
    {
        var signatures = Signatures(best.Select(shape => shape.Method));
        return source.Error("FP3003", "The call is ambiguous between applicable overloads.", position,
            expectation: new("overload", signatures, "ambiguous call"), related: CandidateNotes(signatures));
    }

    private CompilationFailure AddressError(ArgumentModeSyntax syntax, bool addressable)
    {
        var requirement = syntax.Mode == "in" ? "addressable storage" : "writable addressable storage";
        return source.Error("FP3028", $"The '{syntax.Mode}' argument requires {requirement}.", syntax.Value.SpanStart,
            ExpressionLength(syntax.Value), expectation: new("storage", [requirement], addressable ? "read-only storage" : "non-addressable expression"),
            help: syntax.Mode == "in" ? "Bind the value to a typed local before passing it with 'in'." :
                $"Use a typed writable variable for the '{syntax.Mode}' argument.");
    }

    private static int ExpressionLength(ExpressionSyntax syntax) => Math.Max(1, syntax.SpanEnd - syntax.SpanStart);

    private static DiagnosticParameter ExpectedParameter(CallShape shape, int index)
    {
        var name = shape.Method.Parameter(shape.Slots[index]).Name;
        var mode = CallArgumentMode(shape, index);
        var type = shape.Types[index];
        return new(name.Length == 0 ? null : name, TypeFacts.Display(SpanSource(type)),
            mode is "ref" or "out" or "in" ? mode : "value",
            type.Definition?.FullName == "System.Span`1" ? "exact array element type" : null);
    }

    private static string ParameterDescription(DiagnosticParameter parameter) => PassingType(parameter.Mode, parameter.Type) +
        (parameter.Requirement is null ? "" : $" ({parameter.Requirement})");
    private static string PassingType(string mode, string type) => mode == "value" ? type : $"{mode} {type}";
    private static string ConstraintDescription(ReferenceCatalog.GenericConstraintFailure failure)
        => $"Type argument '{failure.Argument}' for '{failure.Parameter}' must satisfy '{failure.Requirement}'.";
    private static string CallSignature(ImportedMethod method) => MethodText(method, method.Owner == TypeSymbol.Object ? method.Name : $"{method.Owner}.{method.Name}");
    private static string[] Signatures(IEnumerable<ImportedMethod> methods)
        => methods.Select(CallSignature).Distinct().Order(StringComparer.Ordinal).Take(CallDiagnosticLimit).ToArray();
    private static DiagnosticNote[] CandidateNotes(IEnumerable<string> signatures)
        => signatures.Select(signature => new DiagnosticNote($"Candidate: {signature}.")).ToArray();
}
