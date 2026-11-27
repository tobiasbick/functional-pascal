// [Documentation](../../../docs/ui/gui/smart-inputs/money-field.md)
using System.Globalization;

namespace Fpas.Ui.Gui.Cui.Tests.Inputs;

/// <summary>Checks exact decimal amounts and native text-editing lifecycle.</summary>
public sealed class CuiMoneyFieldTests
{
    /// <summary>Retains values outside binary floating-point precision and the full decimal range.</summary>
    /// <param name="renderer">Native rendering backend.</param>
    [GuiTheory]
    [InlineData(CuiRenderer.Software)]
    [InlineData(CuiRenderer.OpenGl)]
    public void DecimalAmountsStayExact(CuiRenderer renderer)
    {
        using var field = new Field(renderer);
        field.Draw(focus: true);
        foreach (var (text, places, expected) in new (string, int, decimal)[] {
            ("0.30", 2, 0.1m + 0.2m),
            ("9,007,199,254,740,993.01", 2, 9007199254740993.01m),
            ("79228162514264337593543950335", 0, decimal.MaxValue),
            ("-79228162514264337593543950335", 0, decimal.MinValue),
            ("0.0000000000000000000000000001", 28, .0000000000000000000000000001m),
            ("7922816251426433759354395033.5", 1, 7922816251426433759354395033.5m),
            ("-0.00", 2, 0m) })
        {
            field.Model = 0; field.Places = places; field.Draw(focus: true);
            field.Model = field.Replace(text);
            Assert.Equal(expected, field.Model);
            Assert.Equal(expected, field.Draw());
        }
    }

    /// <summary>Rejects overflow, fractional truncation, silent decimal rounding and malformed groups.</summary>
    /// <param name="text">Invalid or incomplete draft.</param>
    /// <param name="places">Maximum accepted fractional digits.</param>
    [GuiTheory]
    [InlineData("79228162514264337593543950336", 0)]
    [InlineData("-79228162514264337593543950336", 0)]
    [InlineData("7922816251426433759354395033.51", 2)]
    [InlineData("9999999999999999999999999999.99", 2)]
    [InlineData("0.00000000000000000000000000001", 28)]
    [InlineData("1.234", 2)]
    [InlineData("1,23", 2)]
    [InlineData("1,,234", 2)]
    [InlineData("1e3", 2)]
    [InlineData("NaN", 2)]
    [InlineData("Infinity", 2)]
    [InlineData("12x", 2)]
    [InlineData("12.", 2)]
    [InlineData("-", 2)]
    [InlineData("+", 2)]
    [InlineData("", 2)]
    public void InvalidDraftsNeverChangeTheAmount(string text, int places)
    {
        using var field = new Field(CuiRenderer.Software) { Model = 7m, Places = places };
        field.Draw(focus: true);
        Assert.Equal(7m, field.Replace(text));
        Assert.Equal(7m, field.Draw());
        Assert.Equal(7m, field.Draw(key: 2));
        Assert.Equal(7m, field.Draw());
        Assert.Equal(8m, field.Replace("8"));
    }

    /// <summary>Retains incomplete drafts while editing and reconciles rejected/external values.</summary>
    [GuiFact]
    public void DraftsAcceptAcknowledgementsAndRejectExternalConflicts()
    {
        using var field = new Field(CuiRenderer.Software) { Model = 12m };
        field.Draw(focus: true);
        Assert.Equal(12m, field.Replace("-"));
        field.Model = field.Draw(text: "1");
        Assert.Equal(-1m, field.Model);
        field.Model = field.Draw(text: ".");
        Assert.Equal(-1m, field.Model);
        field.Model = field.Draw(text: "2");
        Assert.Equal(-1.2m, field.Model);
        Assert.Equal(-1.2m, field.Draw());
        Assert.Equal(99m, field.Replace("99"));
        // The application rejects 99 by retaining the confirmed -1.2 model.
        Assert.Equal(-1.2m, field.Draw());
        field.Model = 123.45m;
        Assert.Equal(123.45m, field.Draw());
        Assert.Equal(12.34m, field.Replace("12.34"));
    }

    /// <summary>Preserves CUI's undo history after the application acknowledges a decimal proposal.</summary>
    [GuiFact]
    public void AcknowledgedAmountsRetainNativeUndo()
    {
        using var field = new Field(CuiRenderer.Software) { Model = 123.45m };
        field.Draw(focus: true);
        field.Model = field.Replace("12.34");
        Assert.Equal(12.34m, field.Draw());
        Assert.Equal(123.45m, field.Draw(key: 40, mods: 2));
    }

    /// <summary>Opening the editor removes digit grouping and lets the user type into an initial zero.</summary>
    /// <param name="pointer">Use a pointer click to open the editor.</param>
    [GuiTheory]
    [InlineData(false)]
    [InlineData(true)]
    public void FocusStartsWithPlainDigits(bool pointer)
    {
        using var field = new Field(CuiRenderer.Software) { Model = 1249.50m };
        field.Draw();
        if (pointer)
        {
            var bounds = field.Window.GetControlRectangle("amount");
            field.Draw(point: bounds, pressed: true);
            field.Draw(point: bounds, pressed: false);
            field.Draw();
        }
        else field.Draw(focus: true);
        Assert.True(field.Window.ControlFocused("amount"));
        field.Draw(key: 11); // Home
        field.Draw(key: 8); // Right
        Assert.Equal(149.50m, field.Draw(key: 5)); // Delete the second digit.

        field.Model = 0m;
        field.Draw(focus: true);
        Assert.Equal(7m, field.Draw(text: "7"));
    }

    /// <summary>Formats invariantly, retires drafts without inspection and survives disappearance/discard.</summary>
    /// <param name="track">Whether inspection is requested.</param>
    [GuiTheory]
    [InlineData(true)]
    [InlineData(false)]
    public void CultureDisabledAndRemovalRetainTheModel(bool track)
    {
        var original = CultureInfo.CurrentCulture;
        try
        {
            CultureInfo.CurrentCulture = CultureInfo.GetCultureInfo("de-DE");
            using var field = new Field(CuiRenderer.Software) { Model = 1234.56m, Track = track };
            field.Draw(focus: true);
            Assert.Equal(2345.67m, field.Replace("2,345.67"));
            Assert.Equal(field.Model, field.Draw());
            Assert.Equal(field.Model, field.Replace("99", enabled: false));
            field.Draw(focus: true);
            Assert.Equal(field.Model, field.Replace("-"));
            Assert.True(field.Window.NextFrame()); field.Window.Text("Removed"); field.Window.Present();
            field.Draw(focus: true);
            Assert.Equal(12.34m, field.Replace("12.34"));
            Assert.True(field.Window.NextFrame()); field.Window.DiscardFrame();
            Assert.Equal(field.Model, field.Draw(focus: true));
            Assert.Equal(0.10m, field.Replace("0.10"));
        }
        finally { CultureInfo.CurrentCulture = original; }
    }

    /// <summary>Rejects lossy model values before native state changes and supports maximum-value formatting.</summary>
    [GuiFact]
    public void ModelPrecisionIsAnExplicitContract()
    {
        using var field = new Field(CuiRenderer.Software);
        field.Model = .001m;
        Assert.Throws<ArgumentException>(() => field.Draw());
        field.Window.DiscardFrame();
        field.Model = decimal.MaxValue;
        field.Places = 28;
        Assert.Equal(decimal.MaxValue, field.Draw());
    }

    private sealed class Field(CuiRenderer renderer) : IDisposable
    {
        internal readonly CuiWindow Window = new("Decimal money", 640, 260, renderer, continuous: true);
        internal decimal Model;
        internal int Places = 2;
        internal bool Track = true;

        internal decimal Replace(string text, bool enabled = true)
        {
            Draw(key: 15, mods: 2, enabled: enabled);
            if (text.Length == 0) return Draw(key: 4, enabled: enabled);
            return Draw(text: text, enabled: enabled);
        }

        internal decimal Draw(bool focus = false, string? text = null, int key = 0, int mods = 0, bool enabled = true,
            CuiRectangle? point = null, bool? pressed = null)
        {
            Assert.True(Window.NextFrame());
            if (point is { } bounds) Window.MovePointer(bounds.X + bounds.Width / 2, bounds.Y + bounds.Height / 2);
            else Window.MovePointer(-1, -1);
            if (pressed.HasValue) Window.SetLeftButton(pressed.Value);
            if (key != 0) { Window.SetKey(key, true, mods); Window.SetKey(key, false, mods); }
            if (text is not null) { Window.SetKey(0, false); Window.InputText(text); }
            Window.BeginColumn();
            decimal result = Window.MoneyField("amount", Model, "Amount ## literal", "€", "EUR", "Exact decimal",
                Places, 2, 400, enabled, focus, Track);
            Window.EndColumn(); Window.Present();
            return result;
        }

        public void Dispose() => Window.Dispose();
    }
}
