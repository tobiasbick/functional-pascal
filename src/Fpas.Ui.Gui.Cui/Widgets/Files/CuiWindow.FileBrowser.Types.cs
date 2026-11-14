using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Files;

/// <summary>Current native folder and selection.</summary>
/// <param name="Folder">Current folder.</param>
/// <param name="Selected">Selected entry, or null.</param>
/// <param name="Picked">Activated file, or null.</param>
/// <param name="Error">The folder was unreadable.</param>
/// <param name="Parts">Availability of browser parts.</param>
public readonly record struct CuiFileBrowserResult(string Folder, string? Selected, string? Picked, bool Error, bool[] Parts);
