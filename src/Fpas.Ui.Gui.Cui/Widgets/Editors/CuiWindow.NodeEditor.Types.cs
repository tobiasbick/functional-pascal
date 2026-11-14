using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Editors;

/// <summary>Native data for NodeEditor.</summary>
/// <param name="X">X supplied to CUI.</param>
/// <param name="Y">Y supplied to CUI.</param>
/// <param name="Zoom">Zoom supplied to CUI.</param>
[System.Runtime.InteropServices.StructLayout(System.Runtime.InteropServices.LayoutKind.Sequential)]
public readonly record struct CuiNodeView(float X, float Y, float Zoom);
