using System.Globalization;
using System.Numerics;
using Fpas.Debugging.Core;
using Fpas.Debugging.Evaluation;

namespace Fpas.Debugging.Native;

internal sealed partial class CoreClrSession
{
    private sealed partial class EvaluationContext
    {
        public DebugBinding Index(DebugBinding receiver, DebugBinding[] indices)
        {
            ScalarOperations.Available(receiver); Read(indices.Length);
            var selected = indices.Select(IndexNumber).ToArray();
            if (receiver.Kind == "null") throw new DebugException("null_reference", "A null receiver cannot be indexed.");
            if (receiver.Type == "string")
            {
                if (selected.Length != 1) throw new DebugException("invalid_expression", "String indexing requires one UTF-16 index.");
                var text = ScalarOperations.StoredString(receiver); var index = selected[0];
                if (index < 0 || index >= text.Length) throw new DebugException(receiver.Truncated && index >= text.Length ? "truncated_value" : "index_out_of_range", "The string index is outside retained UTF-16 content.");
                return ScalarOperations.Literal(new("literal", "#" + ((int)text[index]).ToString(CultureInfo.InvariantCulture), 0)) with { Constant = false };
            }
            if (receiver.Lengths is not { } lengths || receiver.LowerBounds is not { } lower || selected.Length != lengths.Length)
                throw new DebugException("invalid_expression", "Array indexing requires the runtime rank's number of integral indexes.");
            long flat = 0;
            for (var dimension = 0; dimension < lengths.Length; dimension++)
            {
                var offset = (long)selected[dimension] - lower[dimension];
                if (offset < 0 || offset >= lengths[dimension]) throw new DebugException("index_out_of_range", "The index is outside the array's actual bounds.");
                flat = checked(flat * lengths[dimension] + offset);
            }
            if (receiver.Reference is not { } reference) throw new DebugException("unavailable", "The array has no available value reference.");
            Read(1); var page = session.ValueChildren(reference, checked((int)flat), 1, "raw");
            return page.Items.SingleOrDefault() ?? throw new DebugException("unavailable", "The stored array element is unavailable.");
        }

        private static int IndexNumber(DebugBinding value)
        {
            ScalarOperations.Available(value);
            if (value.Type == "char") return ScalarOperations.StoredString(value)[0];
            if (value.Type is not ("sbyte" or "byte" or "short" or "ushort" or "int"))
                throw new DebugException("unsupported_expression", "An index must convert implicitly to int.");
            var number = BigInteger.Parse(value.Scalar!, CultureInfo.InvariantCulture);
            if (number < int.MinValue || number > int.MaxValue) throw new DebugException("index_out_of_range", "The index is outside Int32 bounds.");
            return (int)number;
        }
    }
}
