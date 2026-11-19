// [Documentation](../../../docs/ui/gui/cui.md#theme-and-resources)
using Fpas.Ui.Gui.Cui.Interop;
using Fpas.Ui.Gui.Cui.Resources;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    private int[]? defaultFonts;
    private int[]? roleFonts;

    private void CheckResourceChange()
    {
        CheckOpen();
        if (inFrame) throw new InvalidOperationException("Replace CUI resources between frames, after presentation.");
    }

    /// <summary>Copies a trusted TrueType/OpenType face into this window's context.</summary>
    /// <param name="data">Complete font bytes, no longer retained by managed code after the call.</param>
    /// <returns>A context-owned face, or null when CUI rejects the font.</returns>
    public unsafe CuiFontFace? LoadFont(byte[] data)
    {
        CheckResourceChange();
        ArgumentNullException.ThrowIfNull(data);
        fixed (byte* bytes = data)
        {
            int id = NativeMethods.FontLoad(context, bytes, (nuint)data.Length);
            return id < 0 ? null : new(this, id);
        }
    }

    /// <summary>Replaces a native font role, without freeing previously loaded faces.</summary>
    /// <param name="role">Body=0, medium=1, display=2, mono=3, icons=4.</param>
    /// <param name="face">A face from this window, or null to restore the built-in face.</param>
    public void SetFontRole(int role, CuiFontFace? face)
    {
        CheckResourceChange();
        ArgumentOutOfRangeException.ThrowIfNegative(role);
        ArgumentOutOfRangeException.ThrowIfGreaterThan(role, 4);
        if (face is not null) CheckFace(face);
        defaultFonts ??= Enumerable.Range(0, 5).Select(i => NativeMethods.FontRole(context, i)).ToArray();
        roleFonts ??= (int[])defaultFonts.Clone();
        int id = face?.Id ?? defaultFonts[role];
        for (int i = 0; i < roleFonts.Length; i++)
            if (i != role && (role == 4 || i == 4) && roleFonts[i] == id)
                throw new ArgumentException("CUI's icon role requires a face dedicated to icons.", nameof(face));
        NativeMethods.FontSetRole(context, role, id);
        roleFonts[role] = id;
    }

    /// <summary>Adds a missing-glyph fallback to a loaded face. CUI supports four direct fallbacks.</summary>
    /// <param name="face">The face whose missing glyphs should fall back.</param>
    /// <param name="fallback">Another face from this window; duplicates and self-fallbacks are rejected.</param>
    public void AddFontFallback(CuiFontFace face, CuiFontFace fallback)
    {
        CheckResourceChange(); CheckFace(face); CheckFace(fallback);
        if (face == fallback || face.Fallbacks.Contains(fallback.Id))
            throw new ArgumentException("A fallback must be another, previously unregistered face.", nameof(fallback));
        if (face.Fallbacks.Count == 4) throw new InvalidOperationException("CUI supports four direct fallback faces.");
        NativeMethods.FontFallback(context, face.Id, fallback.Id);
        face.Fallbacks.Add(fallback.Id);
    }

    private void CheckFace(CuiFontFace face)
    {
        ArgumentNullException.ThrowIfNull(face);
        if (face.Owner != this) throw new ArgumentException("The font belongs to another CUI window.", nameof(face));
    }

    /// <summary>Replaces the native emoji-rendering font with a context-owned copy.</summary>
    /// <param name="data">Complete trusted emoji-font bytes.</param>
    /// <returns>Whether CUI accepted the font, independently of glyph coverage. A parse failure can clear the previous emoji font.</returns>
    public unsafe bool LoadEmojiFont(byte[] data)
    {
        CheckResourceChange(); ArgumentNullException.ThrowIfNull(data);
        fixed (byte* bytes = data) return NativeMethods.EmojiLoad(context, bytes, (nuint)data.Length) != 0;
    }

    /// <summary>Asks CUI to load the platform's standard color-emoji font.</summary>
    /// <returns>Whether CUI found and loaded a supported system font.</returns>
    public bool LoadSystemEmojiFont()
    {
        CheckResourceChange();
        return NativeMethods.EmojiSystem(context) != 0;
    }

    /// <summary>Queries native emoji support in the currently loaded font.</summary>
    /// <param name="text">A Unicode emoji sequence.</param>
    /// <returns>Whether CUI can render that sequence as color emoji.</returns>
    public bool SupportsEmoji(string text)
    {
        CheckOpen(); Utf8.Validate(text, nameof(text));
        return NativeMethods.EmojiSupported(context, text) != 0;
    }
}
