using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Planning;

/// <summary>A native card move measured without the card at its old location.</summary>
/// <param name="Key">Card identity.</param>
/// <param name="FromColumn">Previous column index.</param>
/// <param name="ToColumn">New column index.</param>
/// <param name="Index">Insertion index in the new column.</param>
[StructLayout(LayoutKind.Sequential)]
public readonly record struct CuiKanbanMove(int Key, int FromColumn, int ToColumn, int Index);
