namespace Fpas.Ui.Gui.Cui.Interop;

internal static class CountFormat
{
    // CUI receives integer varargs: accept only literal percent signs and bounded %d slots.
    internal static string? Check(string value, int arguments)
    {
        Utf8.Validate(value, nameof(value));
        int slots = 0;
        for (int index = 0; index < value.Length; index++)
        {
            if (value[index] != '%') continue;
            if (++index == value.Length || value[index] != '%' && (value[index] != 'd' || ++slots > arguments))
                throw new ArgumentException($"Count formats allow at most {arguments} %d slots and literal %%.", nameof(value));
        }
        return value.Length == 0 ? null : value;
    }
}
