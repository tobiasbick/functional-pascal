using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Files;

/// <summary>File paths or a browse activation from CUI's dropzone.</summary>
/// <param name="Paths">Accepted paths copied out of the current native frame.</param>
/// <param name="Browse">Whether the user requested a file dialog.</param>
public readonly record struct CuiDropzoneResult(string[] Paths, bool Browse);
