using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

public sealed partial class CuiWindow
{
    /// <summary>Opens a native carousel for the supplied model selection.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="count">Number of slides.</param>
    /// <param name="index">Selected slide index.</param>
    /// <param name="height">Slide height.</param>
    /// <param name="gap">Slide gap.</param>
    /// <param name="advance">Seconds until automatic advance.</param>
    /// <param name="flags">Enabled, hidden arrows, hidden dots and loop bits.</param>
    /// <param name="focus">Requests native keyboard focus.</param>
    public void BeginCarousel(string id, int count, int index, float height, float gap, float advance, int flags, bool focus)
    {
        CheckFrame();
        if (count is < 1 or > 64 || index < 0 || index >= count) throw new ArgumentOutOfRangeException(nameof(index));
        CheckFeedbackNumber(height, nameof(height));
        CheckFeedbackNumber(gap, nameof(gap));
        CheckFeedbackNumber(advance, nameof(advance));
        int root = NativeMethods.CarouselBegin(context, Stable(id), count, index, height, gap, advance, flags, focus ? 1 : 0);
        layouts.Add(new(Layout.Carousel, Node: root, Data: (Count: count, Enabled: (flags & 1) != 0)));
    }

    /// <summary>Opens the next carousel slide.</summary>
    public void BeginCarouselSlide()
    {
        RequireLayout(Layout.Carousel);
        NativeMethods.CarouselSlideBegin(context);
        layouts.Add(new(Layout.CarouselSlide));
    }

    /// <summary>Closes the current slide.</summary>
    public void EndCarouselSlide()
    {
        RequireLayout(Layout.CarouselSlide);
        NativeMethods.CarouselSlideEnd(context);
        layouts.RemoveAt(layouts.Count - 1);
    }

    /// <summary>Closes the carousel and returns the native selection.</summary>
    /// <param name="part">Optional tracking names for the viewport and dots.</param>
    /// <returns>The proposed slide index.</returns>
    public unsafe int EndCarousel(Func<int, string>? part)
    {
        var scope = RequireLayout(Layout.Carousel);
        var (slides, enabled) = ((int, bool))scope.Data!;
        int count = slides + 1;
        int* nodes = stackalloc int[count];
        int index = NativeMethods.CarouselEnd(context, scope.Node, enabled ? 1 : 0, nodes);
        layouts.RemoveAt(layouts.Count - 1);
        for (int i = 0; i < count; i++)
            if (nodes[i] >= 0 && part is not null) TrackControl(part(i), nodes[i]);
        return index;
    }
}
