using System.Buffers;
using System.Text;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Inputs;

/// <summary>Presentation and capacity shared by native search and multiline editors.</summary>
/// <param name="Label">Optional caption above the editor.</param>
/// <param name="Placeholder">Text displayed while empty.</param>
/// <param name="Icon">Optional leading glyph.</param>
/// <param name="Hint">Supporting text below the editor.</param>
/// <param name="Error">Validation error that replaces the hint.</param>
/// <param name="Prefix">Muted text before the value.</param>
/// <param name="Suffix">Muted text after the value.</param>
/// <param name="Size">Native medium, small or large size ordinal.</param>
/// <param name="ReadOnly">Allow focus and selection while preventing edits.</param>
/// <param name="MaxUtf8Bytes">Maximum value size excluding the terminator.</param>
public sealed record CuiEditorOptions(string Label, string Placeholder, string Icon, string Hint, string Error,
    string Prefix, string Suffix, int Size, bool ReadOnly, int MaxUtf8Bytes);
