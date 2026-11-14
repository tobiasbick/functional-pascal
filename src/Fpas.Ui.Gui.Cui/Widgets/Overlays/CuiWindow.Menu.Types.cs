using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Overlays;

/// <summary>The kind of a native menu row.</summary>
public enum CuiMenuEntryKind
{
    /// <summary>A command.</summary>
    Action,
    /// <summary>A section caption.</summary>
    Label,
    /// <summary>A separator.</summary>
    Separator
}

/// <summary>A literal, keyed native menu row.</summary>
/// <param name="Kind">Command, label or separator.</param>
/// <param name="Key">Stable command key; ignored for noninteractive rows.</param>
/// <param name="Label">Literal caption.</param>
/// <param name="Icon">Optional icon-font glyph.</param>
/// <param name="Shortcut">Display-only shortcut hint.</param>
/// <param name="Enabled">Whether the command can be chosen.</param>
/// <param name="Danger">Use the native danger color.</param>
/// <param name="Checked">Show a check mark.</param>
public sealed record CuiMenuEntry(CuiMenuEntryKind Kind, string Key, string Label, string Icon = "",
    string Shortcut = "", bool Enabled = true, bool Danger = false, bool Checked = false);

/// <summary>The native menu's current interaction result.</summary>
/// <param name="Choice">Chosen entry index, or minus one.</param>
/// <param name="Open">Whether the popup accepts input.</param>
/// <param name="Highlighted">Highlighted entry index, or minus one.</param>
public readonly record struct CuiMenuResult(int Choice, bool Open, int Highlighted);
