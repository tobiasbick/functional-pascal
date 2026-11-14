using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Collections;

/// <summary>A native filter field.</summary>
/// <param name="Name">Literal heading.</param>
/// <param name="Icon">Optional CUI glyph.</param>
/// <param name="Values">Options in mask bit order.</param>
public readonly record struct CuiFilterField(string Name, string Icon, string[] Values);

/// <summary>The native filter bar's output.</summary>
/// <param name="Changed">Whether the native widget changed a model value.</param>
/// <param name="Search">Resulting search, or null when hidden.</param>
/// <param name="Parts">Whether the search, add, clear and individual chips exist in the frame.</param>
public readonly record struct CuiFilterResult(bool Changed, string? Search, bool[] Parts);
