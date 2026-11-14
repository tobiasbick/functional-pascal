using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Navigation;

/// <summary>Native path activation and text submission.</summary>
/// <param name="Segment">Activated segment, or minus one.</param>
/// <param name="Text">Submitted path, or null.</param>
/// <param name="MenuSegment">Menu parent segment, or minus one.</param>
/// <param name="Child">Activated menu item, or minus one.</param>
/// <param name="Parts">Availability of native parts.</param>
public readonly record struct CuiPathResult(int Segment, string? Text, int MenuSegment, int Child, bool[] Parts);
