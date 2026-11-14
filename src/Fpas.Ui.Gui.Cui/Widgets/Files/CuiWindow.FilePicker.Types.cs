using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Files;

/// <summary>The native picker closure and its visible parts.</summary>
/// <param name="Closed">The picker was dismissed or confirmed this frame.</param>
/// <param name="Path">Chosen path, or null on dismissal.</param>
/// <param name="Parts">Availability of the dialog and its controls.</param>
public readonly record struct CuiFilePickerResult(bool Closed, string? Path, bool[] Parts);
