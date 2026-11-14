using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Canvas;

/// <summary>Native data for FormatBar.</summary>
/// <param name="Color">Color supplied to CUI.</param>
/// <param name="Bold">Bold supplied to CUI.</param>
/// <param name="Underline">Underline supplied to CUI.</param>
/// <param name="Link">Link supplied to CUI.</param>
/// <param name="Style">Style supplied to CUI.</param>
/// <param name="Size">Size supplied to CUI.</param>
/// <param name="Alignment">Alignment supplied to CUI.</param>
/// <param name="List">List supplied to CUI.</param>
[System.Runtime.InteropServices.StructLayout(System.Runtime.InteropServices.LayoutKind.Sequential)]
public record struct CuiTextFormat(int Color, byte Bold, byte Underline, byte Link, int Style, float Size, int Alignment, int List);

/// <summary>Native data for FormatBar.</summary>
/// <param name="Name">Name supplied to CUI.</param>
/// <param name="Size">Size supplied to CUI.</param>
/// <param name="Preview">Preview supplied to CUI.</param>
/// <param name="Font">Font supplied to CUI.</param>
public sealed record CuiTextStyle(string Name, float Size, float Preview, int Font);
