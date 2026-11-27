// [Documentation](../../../../docs/ui/gui/smart-inputs/money-field.md)
using System.Globalization;

namespace Fpas.Ui.Gui.Cui.Widgets.Inputs;

// Decimal parsing must reject implicit CLR rounding of oversized coefficients.
internal static class MoneyAmounts
{
    private static readonly CultureInfo Culture = CultureInfo.InvariantCulture;
    private const NumberStyles Style = NumberStyles.AllowLeadingSign | NumberStyles.AllowDecimalPoint | NumberStyles.AllowThousands;

    internal static void Validate(decimal value, int places)
    {
        if (places is < 0 or > 28) throw new ArgumentOutOfRangeException(nameof(places));
        if (decimal.Round(value, places) != value)
            throw new ArgumentException("The amount must fit the selected fractional precision without rounding.", nameof(value));
    }

    internal static string Format(decimal value, int places, bool editing = false)
    {
        if (editing && value == 0) return "";
        return value.ToString((editing ? "F" : "N") + places.ToString(Culture), Culture);
    }

    internal static bool TryParse(string text, int places, out decimal value)
    {
        value = 0;
        ReadOnlySpan<char> raw = text.AsSpan();
        bool negative = raw.Length > 0 && raw[0] == '-';
        if (raw.Length > 0 && raw[0] is '+' or '-') raw = raw[1..];
        int dot = raw.IndexOf('.');
        ReadOnlySpan<char> whole = dot < 0 ? raw : raw[..dot];
        ReadOnlySpan<char> fraction = dot < 0 ? [] : raw[(dot + 1)..];
        if (whole.IsEmpty || fraction.Length > places || dot >= 0 && fraction.IsEmpty) return false;
        int digits = 0, group = 0;
        bool grouped = false;
        foreach (char c in whole)
        {
            if (c == ',')
            {
                if (group == 0 || (!grouped && group > 3) || grouped && group != 3) return false;
                grouped = true; group = 0;
            }
            else if (c is >= '0' and <= '9') { digits++; group++; }
            else return false;
        }
        if (digits > 29 || grouped && group != 3) return false;
        foreach (char c in fraction) if (c is < '0' or > '9') return false;
        if (!decimal.TryParse(text, Style, Culture, out value)) return false;
        string integers = whole.ToString().Replace(",", "", StringComparison.Ordinal).TrimStart('0');
        if (integers.Length == 0) integers = "0";
        string canonical = (negative && value != 0 ? "-" : "") + integers +
            (dot < 0 ? "" : "." + fraction.ToString());
        return string.Equals(canonical, value.ToString("F" + fraction.Length.ToString(Culture), Culture), StringComparison.Ordinal);
    }
}
