using Fpas.Ui.Gui.Cui.Imaging;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    private readonly HashSet<CuiImage> images = [];
    private readonly List<(nuint Texture, long LastDrawnFrame)> retiredTextures = [];
    private long imageFrame;

    /// <summary>Uploads pixels from an array to a texture owned by this window on its UI thread.</summary>
    /// <param name="pixelWidth">Positive width in pixels.</param>
    /// <param name="pixelHeight">Positive height in pixels.</param>
    /// <param name="rgba">Exactly four tightly packed RGBA bytes per pixel. CUI copies the pixels during the call.</param>
    /// <returns>The owned image. Dispose it when it is no longer needed.</returns>
    public CuiImage CreateImage(int pixelWidth, int pixelHeight, byte[] rgba)
    {
        ArgumentNullException.ThrowIfNull(rgba);
        return CreateImage(pixelWidth, pixelHeight, rgba.AsSpan());
    }

    /// <summary>Uploads pixels to a texture owned by this window on its UI thread.</summary>
    /// <param name="pixelWidth">Positive width in pixels.</param>
    /// <param name="pixelHeight">Positive height in pixels.</param>
    /// <param name="rgba">Exactly four tightly packed RGBA bytes per pixel. CUI copies the pixels during the call.</param>
    /// <returns>The owned image. Dispose it when it is no longer needed.</returns>
    public unsafe CuiImage CreateImage(int pixelWidth, int pixelHeight, ReadOnlySpan<byte> rgba)
    {
        CheckOpen();
        ArgumentOutOfRangeException.ThrowIfNegativeOrZero(pixelWidth);
        ArgumentOutOfRangeException.ThrowIfNegativeOrZero(pixelHeight);
        if (rgba.Length % 4 != 0 || (long)pixelWidth * pixelHeight != rgba.Length / 4)
            throw new ArgumentException("The pixel buffer must contain exactly four bytes per pixel.", nameof(rgba));
        nuint texture;
        fixed (byte* pixels = rgba)
            texture = NativeMethods.CreateImage(app, pixelWidth, pixelHeight, pixels);
        if (texture == 0) throw new InvalidOperationException("CUI could not upload the image texture.");
        try
        {
            var image = new CuiImage(this, pixelWidth, pixelHeight, texture);
            images.Add(image);
            return image;
        }
        catch
        {
            NativeMethods.DestroyImage(context, texture);
            throw;
        }
    }

    /// <summary>Draws an image from this window in the current frame.</summary>
    /// <param name="image">A live image created by this window.</param>
    /// <param name="width">Positive finite display width in logical units.</param>
    /// <param name="height">Positive finite display height in logical units.</param>
    public void Image(CuiImage image, float width, float height)
    {
        CheckFrame();
        ArgumentNullException.ThrowIfNull(image);
        if (!ReferenceEquals(image.Owner, this)) throw new ArgumentException("The image belongs to another CUI window.", nameof(image));
        ObjectDisposedException.ThrowIf(image.Texture == 0, image);
        if (!float.IsFinite(width) || width <= 0) throw new ArgumentOutOfRangeException(nameof(width));
        if (!float.IsFinite(height) || height <= 0) throw new ArgumentOutOfRangeException(nameof(height));
        NativeMethods.Image(context, image.Texture, width, height);
        image.LastDrawnFrame = imageFrame;
    }

    internal void ReleaseImage(CuiImage image)
    {
        if (image.Texture == 0) return;
        CheckOpen();
        // An open frame can paint this texture later, and native resize can replay the last completed frame.
        if (image.LastDrawnFrame > 0 && image.LastDrawnFrame >= imageFrame - 1)
            retiredTextures.Add((image.Texture, image.LastDrawnFrame));
        else
            NativeMethods.DestroyImage(context, image.Texture);
        image.Invalidate();
        images.Remove(image);
    }

    private void AdvanceImageFrame()
    {
        imageFrame++;
        for (int index = retiredTextures.Count - 1; index >= 0; index--)
        {
            var (texture, lastDrawnFrame) = retiredTextures[index];
            if (lastDrawnFrame >= imageFrame - 1) continue;
            NativeMethods.DestroyImage(context, texture);
            retiredTextures.RemoveAt(index);
        }
    }

    private void ReleaseImages()
    {
        if (images.Count == 0 && retiredTextures.Count == 0) return;

        // Finish deferred paint while the textures are still valid. Teardown never presents that frame.
        NativeMethods.FinishFrameForDestroy(app);
        foreach (CuiImage image in images)
        {
            NativeMethods.DestroyImage(context, image.Texture);
            image.Invalidate();
        }
        foreach (var (texture, _) in retiredTextures)
            NativeMethods.DestroyImage(context, texture);
        images.Clear();
        retiredTextures.Clear();
    }
}
