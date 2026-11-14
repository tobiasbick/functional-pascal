using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Editors;

/// <summary>One native inspector row.</summary>
/// <param name="Label">Row label.</param>
/// <param name="Kind">Native value kind.</param>
/// <param name="Integer">Integer, boolean or selection.</param>
/// <param name="Values">Number or vector data, with at most four components.</param>
/// <param name="Text">Text value.</param>
/// <param name="Color">Packed color.</param>
/// <param name="Choices">Enum choices.</param>
/// <param name="Minimum">Numeric minimum.</param>
/// <param name="Maximum">Numeric maximum.</param>
/// <param name="Step">Scrub step.</param>
/// <param name="Digits">Decimal precision.</param>
/// <param name="Suffix">Optional unit.</param>
/// <param name="Placeholder">Empty text hint.</param>
/// <param name="Tooltip">Row tooltip.</param>
/// <param name="Capacity">Text capacity.</param>
/// <param name="Flags">Input and checkbox flags.</param>
public sealed record CuiProperty(string Label, int Kind, int Integer, float[] Values, string? Text, uint Color, string[] Choices, float Minimum, float Maximum, float Step, int Digits, string Suffix, string Placeholder, string Tooltip, int Capacity, int Flags);

/// <summary>One labeled group in the native inspector.</summary>
/// <param name="Title">Group title.</param>
/// <param name="Properties">Property rows.</param>
/// <param name="Open">Expansion state.</param>
/// <param name="Collapsible">Allows collapsing.</param>
public sealed record CuiPropertyGroup(string Title, CuiProperty[] Properties, bool Open, bool Collapsible);

/// <summary>A native inspector proposal.</summary>
/// <param name="Group">Group index.</param>
/// <param name="Row">Property index, or minus one for expansion.</param>
/// <param name="Open">Proposed expansion.</param>
/// <param name="Integer">Proposed scalar integer.</param>
/// <param name="Values">Proposed numeric values.</param>
/// <param name="Text">Proposed text.</param>
/// <param name="Color">Proposed color.</param>
public sealed record CuiPropertyEdit(int Group, int Row, bool Open, int Integer, float[] Values, string? Text, uint Color);

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeProperty
{
    internal const int MaxComponents = 4;

    internal byte* Label, Suffix, Placeholder, Tooltip, Text;
    internal byte** Choices;
    internal int Kind, Integer, Count, Capacity, Digits, Flags, Changed;
    internal float Minimum, Maximum, Step;
    internal fixed float Values[MaxComponents];
    internal uint Rgba;
}

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativePropertyGroup
{
    internal byte* Title;
    internal NativeProperty* Properties;
    internal int Count, Open, Collapsible, Changed;
}
