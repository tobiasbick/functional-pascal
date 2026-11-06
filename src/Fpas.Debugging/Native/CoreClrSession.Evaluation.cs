using Fpas.Debugging.Core;
using Fpas.Debugging.Evaluation;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession
{
    internal EvaluationResult Evaluate(long frame, string expression)
    {
        try
        {
            var syntax = ExpressionParser.Parse(expression);
            lock (gate)
            {
                if (!frames.ContainsKey(frame)) throw new DebugException("stale_reference", "Use a frame from the current stop.");
                var value = new PassiveEvaluator(new EvaluationContext(this, frame)).Evaluate(syntax);
                return new(expression, value with { Name = expression });
            }
        }
        catch (DebugException exception) { return new(expression, Error: new(exception.Code, exception.Message)); }
        catch (OverflowException) { return new(expression, Error: new("overflow", "The scalar operation overflowed its type.")); }
        catch (DivideByZeroException) { return new(expression, Error: new("division_by_zero", "The scalar divisor is zero.")); }
        catch (Exception exception) when (exception is FormatException or BadImageFormatException or IOException)
        { return new(expression, Error: new("unavailable", "The expression's stored value or metadata is unavailable.")); }
    }

    private sealed partial class EvaluationContext(CoreClrSession session, long frame) : IPassiveContext
    {
        private readonly Dictionary<string, DebugBinding?> names = new(StringComparer.Ordinal);
        private int reads = 65536, metadata = 65536;

        private void Read(int count)
        {
            reads -= count;
            if (reads < 0) throw new DebugException("limit_exceeded", "Passive evaluation exceeds 65536 binding/value entries.");
        }
        private void MetadataRead()
        {
            if (--metadata < 0) throw new DebugException("limit_exceeded", "Passive evaluation exceeds 65536 metadata types.");
        }

        private DebugBinding? Find(Func<int, BindingPage> page, string name)
        {
            for (var start = 0; ;)
            {
                var values = page(start); Read(values.Total);
                var found = values.Items.FirstOrDefault(value => value.Name == name);
                if (found is not null) return found;
                start += values.Items.Length;
                if (start >= values.Total) return null;
                if (values.Items.Length == 0) throw new DebugException("limit_exceeded", "A value page could not make progress.");
            }
        }

        public DebugBinding Member(DebugBinding receiver, string name)
        {
            ScalarOperations.Available(receiver);
            if (receiver.Kind == "null") throw new DebugException("null_reference", "A null receiver has no stored fields.");
            if (receiver.Reference is not { } reference) throw new DebugException("unavailable", "The receiver has no expandable stored reference.");
            return Find(start => session.ValueChildren(reference, start, 128, "source"), name)
                ?? throw new DebugException("unresolved_name", $"No stored source field is named '{name}'.");
        }
    }
}
