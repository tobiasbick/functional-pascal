using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
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
        if (Capacity < 1 || Capacity > 4096 || Rules.Length > Capacity) throw new ArgumentOutOfRangeException(nameof(Capacity));
        using var memory = new ChartMemory();
        NativeDevField* fields = PackQueryFields(memory, Fields);
        var storage = new NativeDevRule[Capacity];
        int count = Rules.Length;
        fixed (NativeDevRule* rows = storage)
        {
            for (int i = 0; i < count; i++)
            {
                var rule = Rules[i];
                rows[i].Depth = rule.Depth;
                rows[i].Group = rule.Group ? (byte)1 : (byte)0;
                rows[i].Any = rule.Any ? (byte)1 : (byte)0;
                rows[i].Field = rule.Field;
                rows[i].Operator = rule.Operator;
                WriteDevText(rule.Value, new Span<byte>(rows[i].Value, 64));
            }
            NativeMethods.DevQueryBuilder(context, Stable(Id), fields, Fields.Length, rows, ref count, Capacity, MaxDepth,
                ReadOnly ? 1 : 0, WidthKind, Width, Enabled ? 1 : 0, out var result);
            if (!Enabled || ReadOnly || (result.Flags == 0 && count == Rules.Length)) return Rules;
            var next = new CuiDevRule[count];
            for (int i = 0; i < count; ++i)
                next[i] = new(rows[i].Depth, rows[i].Group != 0, rows[i].Any != 0, rows[i].Field, rows[i].Operator,
                    ReadDevText(new ReadOnlySpan<byte>(rows[i].Value, 64)));
            return next;
        }
    }
}
