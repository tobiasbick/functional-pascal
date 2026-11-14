using System.Text;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Inputs;

/// <summary>A native code edit and CUI's completion event.</summary>
/// <param name="Value">The proposed ASCII code.</param>
/// <param name="Completed">Whether CUI reported a newly completed code this frame.</param>
public readonly record struct CuiOneTimeCodeResult(string Value, bool Completed);
