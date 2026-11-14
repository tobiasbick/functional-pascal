using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Files;

/// <summary>The avatar image state and any file action proposed by CUI.</summary>
/// <param name="Path">The path of a successfully dropped image, if any.</param>
/// <param name="Browse">Whether the user requested a file dialog.</param>
/// <param name="HasImage">Whether CUI displays a decoded image.</param>
public readonly record struct CuiAvatarUploadResult(string? Path, bool Browse, bool HasImage);
