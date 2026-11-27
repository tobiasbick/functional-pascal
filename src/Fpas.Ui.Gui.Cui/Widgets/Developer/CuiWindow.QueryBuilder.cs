// [Documentation](../../../../docs/ui/gui/developer-tools/query-builder.md)
using Fpas.Ui.Gui.Cui.Interop.Developer;
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Tests a record against the native query builder's conditions.</summary>
    /// <param name="fields">Field schema, in record order.</param>
    /// <param name="rules">The model's flattened groups and conditions.</param>
    /// <param name="values">One textual value per field, in schema order.</param>
    /// <returns>Whether the record matches. An empty query accepts every record.</returns>
    public static unsafe bool MatchQuery(CuiDevField[] fields, CuiDevRule[] rules, string[] values)
    {
        ArgumentNullException.ThrowIfNull(fields);
        ArgumentNullException.ThrowIfNull(rules);
        ArgumentNullException.ThrowIfNull(values);
        if (values.Length != fields.Length)
            throw new ArgumentException("Supply one value per query field.", nameof(values));
        foreach (string value in values) Utf8.Validate(value, nameof(values));
        using var memory = new ChartMemory();
        var schema = PackQueryFields(memory, fields);
        var rows = new NativeDevRule[rules.Length];
        fixed (NativeDevRule* data = rows)
        {
            PackQueryRules(rules, data);
            for (int i = 0; i < rules.Length; i++)
            {
                CuiDevRule rule = rules[i];
                if (i == 0 ? rule.Depth != 0 || !rule.Group : rule.Depth == 0 ||
                    rule.Depth > rules[i - 1].Depth + 1 ||
                    rule.Depth > rules[i - 1].Depth && !rules[i - 1].Group)
                    throw new ArgumentException("Expected one root group and ordered descendants.", nameof(rules));
                if (rule.Group) continue;
                if ((uint)rule.Field >= (uint)fields.Length ||
                    (uint)rule.Operator >= (fields[rule.Field].Kind <= 1 ? 6u : 2u))
                    throw new ArgumentException("Invalid query field or operator.", nameof(rules));
            }
            return NativeMethods.MatchQuery(schema, fields.Length, data, rows.Length,
                memory.Labels(values)) != 0;
        }
    }

    /// <summary>Draws CUI's native QueryBuilder widget.</summary>
    /// <param name="Id">Id supplied to the widget.</param>
    /// <param name="Fields">Fields supplied to the widget.</param>
    /// <param name="Rules">Rules supplied to the widget.</param>
    /// <param name="Capacity">Capacity supplied to the widget.</param>
    /// <param name="MaxDepth">Max Depth supplied to the widget.</param>
    /// <param name="ReadOnly">Read Only supplied to the widget.</param>
    /// <param name="WidthKind">Width Kind supplied to the widget.</param>
    /// <param name="Width">Width supplied to the widget.</param>
    /// <param name="Enabled">Enabled supplied to the widget.</param>
    /// <returns>Native edit or action proposals.</returns>
    public unsafe CuiDevRule[] QueryBuilder(string Id, CuiDevField[] Fields, CuiDevRule[] Rules, int Capacity, int MaxDepth, bool ReadOnly, int WidthKind, float Width, bool Enabled)
    {
        CheckFrame();
        CheckExtent(WidthKind, Width, nameof(Width));
        ArgumentNullException.ThrowIfNull(Rules);
        if (Capacity < 1 || Capacity > 4096 || Rules.Length > Capacity) throw new ArgumentOutOfRangeException(nameof(Capacity));
        if (MaxDepth is < 0 or > 64) throw new ArgumentOutOfRangeException(nameof(MaxDepth));
        using var memory = new ChartMemory();
        NativeDevField* fields = PackQueryFields(memory, Fields);
        var storage = new NativeDevRule[Capacity];
        int count = Rules.Length;
        fixed (NativeDevRule* rows = storage)
        {
            PackQueryRules(Rules, rows);
            NativeMethods.DevQueryBuilder(context, Stable(Id), fields, Fields.Length, rows, ref count, Capacity, MaxDepth,
                ReadOnly ? 1 : 0, WidthKind, Width, Enabled ? 1 : 0, out var result);
            if (!Enabled || ReadOnly || (result.Flags == 0 && count == Rules.Length)) return Rules;
            var next = new CuiDevRule[count];
            for (int i = 0; i < count; ++i)
                next[i] = new(rows[i].Depth, rows[i].Group != 0, rows[i].Any != 0, rows[i].Field, rows[i].Operator,
                    ReadDevText(new ReadOnlySpan<byte>(rows[i].Value, 64)));
            // CUI-134 (docs/future/cui-upstream-bugs.md#cui-134): an empty first choice can report an idle edit.
            return next.AsSpan().SequenceEqual(Rules) ? Rules : next;
        }
    }
}
