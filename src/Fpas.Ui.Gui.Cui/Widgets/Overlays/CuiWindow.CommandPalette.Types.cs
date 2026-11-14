using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Overlays;

/// <summary>One CUI palette entry.</summary>
/// <param name="Title">Searchable caption.</param>
/// <param name="Icon">Optional glyph.</param>
/// <param name="Group">Optional group heading.</param>
/// <param name="Shortcut">Optional shortcut hint.</param>
public readonly record struct CuiPaletteCommand(string Title, string Icon, string Group, string Shortcut);

/// <summary>Palette interaction and the indices of rendered command rows.</summary>
/// <param name="Open">Proposed visibility.</param>
/// <param name="Selected">Selected command, or minus one.</param>
/// <param name="Shown">Indices present in the filtered view.</param>
public readonly record struct CuiPaletteResult(bool Open, int Selected, int[] Shown);
