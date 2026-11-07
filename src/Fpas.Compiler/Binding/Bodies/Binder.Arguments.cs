using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Bodies;

internal sealed partial class Binder
{
    private sealed record CallShape(ImportedMethod Method, int[] Slots, bool Expanded, IReadOnlyList<TypeSymbol> Types, int Omitted);

    private CallShape? Shape(ImportedMethod method, IReadOnlyList<ExpressionSyntax> syntax, bool expanded, bool allowMissing = false)
    {
        var count = method.Parameters.Count;
        var variadic = count > 0 && method.Parameter(count - 1).Mode == "params";
        if (expanded && (!variadic || method.Parameters[^1].Kind != TypeKind.Array || method.Parameters[^1].Rank != 1)) return null;
        var slots = new int[syntax.Count];
        var used = new HashSet<int>();
        var next = 0;
        var named = false;
        for (var index = 0; index < syntax.Count; index++)
        {
            int slot;
            if (syntax[index] is NamedArgumentSyntax argument)
            {
                named = true;
                slot = Enumerable.Range(0, count).FirstOrDefault(candidate => method.Parameter(candidate).Name == argument.Name.Text, -1);
                if (slot < 0 || expanded && slot == count - 1) return null;
            }
            else
            {
                if (named) return null;
                slot = expanded && next >= count - 1 ? count - 1 : next++;
            }
            if (slot < 0 || slot >= count || !used.Add(slot) && !(expanded && slot == count - 1)) return null;
            slots[index] = slot;
        }
        var missing = Enumerable.Range(0, count).Where(slot => !used.Contains(slot) && !(expanded && slot == count - 1)).ToArray();
        if (!allowMissing && missing.Any(slot => method.Parameter(slot).Default is null)) return null;
        var types = slots.Select(slot => expanded && slot == count - 1 ? method.Parameters[slot].Element! : method.Parameters[slot]).ToArray();
        return new(method, slots, expanded, types, missing.Length);
    }

    private (CallShape Shape, BoundExpression[] Values) SelectInvocation(IReadOnlyList<ImportedMethod> templates, IReadOnlyList<ExpressionSyntax> syntax,
        bool check, int position, bool inferGenerics, Action? noApplicable = null)
    {
        if (source.Editor is not null) EditorSignatures(templates, syntax, position);
        var incomplete = position == editorInvocation && source.Editor?.Incomplete == true;
        var shapes = templates.SelectMany(method => new[] { Shape(method, syntax, false, incomplete), Shape(method, syntax, true, incomplete) })
            .Where(shape => shape is not null).Select(shape => shape!).ToArray();
        var arguments = syntax.Select(argument => argument is NamedArgumentSyntax named ? named.Value : argument).ToArray();
        var only = shapes.Length == 1 ? shapes[0] : null;
        var inference = only?.Method.MethodArguments is { Count: > 0 } variables ? new TypeInference(references, variables) : null;
        var values = BindInferenceValues(arguments, check, out var argumentFacts, only?.Types, inference);
        var matches = new List<CallShape>();
        var inferenceFailed = false;
        var constraintsFailed = false;
        foreach (var shape in shapes)
        {
            var method = shape.Method;
            if (inferGenerics && method.MethodArguments is { Count: > 0 } generic)
            {
                var solver = new TypeInference(references, generic);
                for (var index = 0; index < values.Length; index++)
                    if (values[index] is BoundAddress address) solver.Add(shape.Types[index], address.Target.Type, InferenceBound.Exact);
                    else if (values[index] is { } value) solver.Add(shape.Types[index], value);
                var fixedTypes = solver.Fix();
                if (fixedTypes is null) { inferenceFailed = true; continue; }
                method = references.CloseMethod(method, fixedTypes);
            }
            if (!references.ValidMethodArguments(method)) { constraintsFailed = true; continue; }
            var closed = Shape(method, syntax, shape.Expanded, incomplete)!;
            if (values.Where((value, index) => !AcceptsArgument(value, arguments[index], closed, index)).Any()) continue;
            matches.Add(closed);
        }
        var best = matches.Where(candidate => !matches.Any(other => BetterCall(other, candidate, values))).ToArray();
        if (source.Editor is not null) EditorSignatures(templates, syntax, position, matches, best.Length == 1 ? best[0] : null);
        if (incomplete) throw source.Error("FP3003", "Editor signature recovery stops at the incomplete call.", position);
        if (best.Length == 0 && !constraintsFailed && !inferenceFailed) noApplicable?.Invoke();
        if (best.Length != 1)
            throw source.Error(best.Length == 0 && constraintsFailed ? "FP3018" : best.Length == 0 && inferenceFailed && matches.Count == 0 ? "FP3026" : "FP3003",
                (best.Length == 0 ? "No applicable overload for the supplied names, argument modes, types, and optional parameters." : "Ambiguous overloads for the supplied arguments.") +
                " Candidates: " + string.Join("; ", templates.Select(method => method.Name + (method.MethodArguments is null ? "" : "<" + string.Join(',', method.MethodArguments) + ">") +
                    "(" + string.Join(", ", method.Parameters.Select((type, index) => method.Parameter(index).Mode + " " + TypeFacts.Display(type))) + ")").Distinct().Take(20)), position);
        var selected = best[0];
        return (selected, BindDeferredArguments(arguments, values, selected.Types, check, argumentFacts));
    }

    private BoundExpression BindInvocation(IReadOnlyList<ImportedMethod> templates, IReadOnlyList<ExpressionSyntax> syntax,
        BoundExpression? receiver, bool check, int position, TypeSymbol? constructed = null, bool inferGenerics = false, Action? noApplicable = null)
    {
        var (selected, bound) = SelectInvocation(templates, syntax, check, position, inferGenerics, noApplicable);
        for (var index = 0; index < syntax.Count; index++)
            if (syntax[index] is NamedArgumentSyntax named) catalog.Analysis?.NamedParameter(source, named.Name, selected.Method, selected.Slots[index]);
        var arguments = syntax.Select(argument => argument is NamedArgumentSyntax named ? named.Value : argument).ToArray();
        if (selected.Method.IsStatic) receiver = null;
        var bindings = new List<BoundBinding>();
        if (receiver is not null)
        {
            if ((receiver.Type.IsValue || receiver.Type.IsParameter) && Writable(receiver))
                receiver = new BoundDereference(Save(new BoundAddress(receiver, "ref"), bindings));
            else receiver = Save(receiver, bindings);
        }
        var saved = new BoundExpression[bound.Length];
        var written = new List<ParameterSymbol>();
        for (var index = 0; index < bound.Length; index++)
        {
            var value = bound[index];
            if (value is BoundAddress address)
            {
                if (address.Mode is "ref" or "out")
                {
                    if (address.Target is BoundParameter parameter) written.Add(parameter.Parameter);
                    InvalidateReference(address.Target);
                }
            }
            else value = Convert(value, selected.Types[index], arguments[index].Position);
            saved[index] = Save(value, bindings);
        }
        var supplied = new BoundExpression[selected.Method.Parameters.Count];
        for (var slot = 0; slot < supplied.Length; slot++)
        {
            var indices = selected.Slots.Select((target, index) => (target, index)).Where(item => item.target == slot).Select(item => item.index).ToArray();
            supplied[slot] = selected.Expanded && slot == supplied.Length - 1
                ? new BoundArrayLiteral(selected.Method.Parameters[slot], indices.Select(index => saved[index]).ToArray())
                : indices.Length == 1 ? saved[indices[0]] : Convert(selected.Method.Parameter(slot).Default!, selected.Method.Parameters[slot], position);
        }
        ForgetCapturedFacts();
        BoundExpression call = constructed is null ? new BoundCall(selected.Method, supplied, receiver, written)
            : new BoundNew(constructed, selected.Method, supplied, written);
        return bindings.Count == 0 ? call : new BoundSequence(bindings, call);
    }

    private bool AcceptsArgument(BoundExpression? value, ExpressionSyntax syntax, CallShape shape, int index)
    {
        var mode = shape.Expanded && shape.Slots[index] == shape.Method.Parameters.Count - 1 ? "" : shape.Method.Parameter(shape.Slots[index]).Mode;
        if (mode is "ref" or "out" or "in")
            return value is BoundAddress address && address.Mode == mode && address.Target.Type == shape.Types[index];
        return value is not BoundAddress && (value is null ? AcceptsDeferred(syntax, shape.Types[index]) : Accepts(value, shape.Types[index]));
    }

    private bool BetterCall(CallShape left, CallShape right, IReadOnlyList<BoundExpression?> values)
    {
        var arguments = values.Select(value => value is BoundAddress address ? address.Target.Type : value?.Type ?? TypeSymbol.Unsupported).ToArray();
        static ImportedMethod View(CallShape shape) => shape.Method with { Parameters = shape.Types };
        var l = View(left); var r = View(right);
        var delegates = values.Select(value => value is BoundDelegate).ToArray();
        if (references.Better(l, r, arguments, delegates, tieBreak: false)) return true;
        if (references.Better(r, l, arguments, delegates, tieBreak: false)) return false;
        if (!left.Types.Select(type => type.Erased).SequenceEqual(right.Types.Select(type => type.Erased))) return false;
        var leftGeneric = left.Method.MethodArguments is { Count: > 0 };
        var rightGeneric = right.Method.MethodArguments is { Count: > 0 };
        if (leftGeneric != rightGeneric) return !leftGeneric;
        if (left.Expanded != right.Expanded) return !left.Expanded;
        if (left.Expanded)
        {
            var leftElements = left.Slots.Count(slot => slot == left.Method.Parameters.Count - 1);
            var rightElements = right.Slots.Count(slot => slot == right.Method.Parameters.Count - 1);
            if (leftElements != rightElements) return leftElements < rightElements;
        }
        if (leftGeneric && left.Method.Parameters.Count == right.Method.Parameters.Count)
        {
            if (ReferenceCatalog.MoreSpecific(left.Method, right.Method)) return true;
            if (ReferenceCatalog.MoreSpecific(right.Method, left.Method)) return false;
        }
        if (left.Omitted == 0 && right.Omitted > 0) return true;
        if (left.Expanded && right.Expanded)
        {
            var leftArray = left.Method.Parameters[^1]; var rightArray = right.Method.Parameters[^1];
            return references.CanAssign(leftArray, rightArray) && !references.CanAssign(rightArray, leftArray);
        }
        return false;
    }

    private BoundExpression BindAddress(ArgumentModeSyntax syntax, bool check)
    {
        var value = syntax.Value switch
        {
            NameExpressionSyntax name => Value(name.Name.Text, name.Position),
            MemberSyntax member => BindMember(member, writing: syntax.Mode != "in", check: check),
            IndexSyntax index => BindIndex(index, check, writing: syntax.Mode != "in"),
            _ => throw source.Error("FP3028", "A managed argument requires addressable storage.", syntax.Position)
        };
        value = value switch { BoundLocal local => new BoundLocal(local.Local), BoundParameter parameter => new BoundParameter(parameter.Parameter), BoundCapture capture => new BoundCapture(capture.Capture), _ => value };
        catalog.Analysis?.Expression(source, syntax.Value, value, syntax.Mode == "out" ? "write" : syntax.Mode == "ref" ? "read_write" : "read");
        var addressable = value is BoundLocal or BoundParameter or BoundCapture or BoundArrayIndex or BoundField;
        if (!addressable || syntax.Mode != "in" && !Writable(value))
            throw source.Error("FP3028", "ref/out arguments require writable storage; ordinary properties are not addressable.", syntax.Position);
        return new BoundAddress(value, syntax.Mode);
    }

    private void InvalidateReference(BoundExpression value)
    {
        var symbol = value switch { BoundLocal local => (object)local.Local, BoundParameter parameter => parameter.Parameter, BoundCapture capture => capture.Capture, _ => null };
        if (symbol is not null) nonNull.Remove(symbol);
    }
}
