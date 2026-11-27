// [Documentation](../../../../docs/ui/gui/smart-inputs/money-field.md)
using Fpas.Ui.Gui.Cui.Interop;
using Fpas.Ui.Gui.Cui.Widgets.Inputs;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    private sealed class MoneyEdit(decimal value, int places)
    {
        internal decimal Expected = value;
        internal int Places = places;
        internal string Draft = MoneyAmounts.Format(value, places);
        internal bool Focused, Invalid;
        internal uint Key;

        internal void Reset(decimal amount, int precision, bool editing)
        {
            Expected = amount; Places = precision;
            Draft = MoneyAmounts.Format(amount, precision, editing);
            Invalid = false;
        }
    }

    private readonly Dictionary<string, MoneyEdit> moneyFields = new(StringComparer.Ordinal);
    private readonly HashSet<string> moneyDrawn = new(StringComparer.Ordinal);

    private void RetireMoneyFields()
    {
        foreach (string id in moneyFields.Keys.Where(id => !moneyDrawn.Contains(id)).ToArray())
        {
            var edit = moneyFields[id];
            moneyFields.Remove(id);
            if (!trackedNodes.ContainsKey(id)) NativeMethods.Forget(context, edit.Key);
        }
    }

    private void DiscardMoneyFields()
    {
        foreach (var edit in moneyFields.Values) NativeMethods.Forget(context, edit.Key);
        moneyFields.Clear(); moneyDrawn.Clear();
    }
}
