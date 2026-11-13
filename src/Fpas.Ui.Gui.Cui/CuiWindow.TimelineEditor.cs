using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

/// <summary>A timeline curve key.</summary>
/// <param name="Curve">Curve data.</param>
/// <param name="Selected">Whether selected.</param>
public sealed record CuiTimelineKey(CuiCurveKey Curve, bool Selected);

/// <summary>A clip on the timeline.</summary>
/// <param name="Start">Start seconds.</param>
/// <param name="End">End seconds.</param>
/// <param name="Offset">Media offset seconds.</param>
/// <param name="Name">Visible name.</param>
/// <param name="Selected">Whether selected.</param>
public sealed record CuiTimelineClip(float Start, float End, float Offset, string Name, bool Selected);

/// <summary>A native timeline track.</summary>
/// <param name="Name">Track name.</param>
/// <param name="Icon">Optional icon.</param>
/// <param name="Depth">Hierarchy depth.</param>
/// <param name="Collapsed">Collapse child tracks.</param>
/// <param name="Muted">Muted flag.</param>
/// <param name="Locked">Editing lock.</param>
/// <param name="Color">Series color.</param>
/// <param name="Keys">Curve keys.</param>
/// <param name="Capacity">Key capacity.</param>
/// <param name="Clips">Track clips.</param>
public sealed record CuiTimelineTrack(string Name, string Icon, int Depth, bool Collapsed, bool Muted, bool Locked, int Color, CuiTimelineKey[] Keys, int Capacity, CuiTimelineClip[] Clips);

/// <summary>Timeline state for one frame.</summary>
/// <param name="Time">Playhead.</param>
/// <param name="Start">Viewport start.</param>
/// <param name="End">Viewport end.</param>
/// <param name="Scroll">Vertical scroll.</param>
/// <param name="Track">Current track.</param>
/// <param name="Key">Current key.</param>
/// <param name="HasRange">Whether the play range is visible.</param>
/// <param name="In">Play range start.</param>
/// <param name="Out">Play range end.</param>
public sealed record CuiTimelineRange(float Time, float Start, float End, float Scroll, int Track, int Key, bool HasRange, float In, float Out);

/// <summary>Proposed timeline state.</summary>
/// <param name="Tracks">Updated tracks.</param>
/// <param name="View">Updated view.</param>
/// <param name="Play">Whether Space requested playback.</param>
public sealed record CuiTimelineEdit(CuiTimelineTrack[] Tracks, CuiTimelineRange View, bool Play);

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeTimelineTrack
{
    internal byte* Name, Icon;
    internal int Depth;
    internal byte Collapsed, Muted, Locked, Color;
    internal CuiCurveKey* Keys;
    internal int KeyCount, KeyCapacity;
    internal byte* KeySelected;
    internal NativeTimelineClip* Clips;
    internal int ClipCount;
}

[StructLayout(LayoutKind.Sequential)]
internal unsafe struct NativeTimelineClip
{
    internal float Start, End, Offset;
    internal byte* Name;
    internal byte Selected;
}

[StructLayout(LayoutKind.Sequential)]
internal struct NativeTimelineView
{
    internal float Time, Start, End, Scroll, In, Out;
    internal int Track, Key, Range;
}

[StructLayout(LayoutKind.Sequential)]
internal struct NativeTimelineOptions
{
    internal float Start, End, Fps, Height, RowHeight, TrackWidth, Width;
    internal int Ruler, WidthKind, Flags;
}

public sealed partial class CuiWindow
{
    /// <summary>Draws a native TimelineEditor.</summary>
    /// <param name="id">Stable widget identity.</param>
    /// <param name="tracks">Ordered tracks and edit data.</param>
    /// <param name="view">Model-owned state.</param>
    /// <param name="start">Sequence start.</param>
    /// <param name="end">Sequence end.</param>
    /// <param name="fps">Frames per second.</param>
    /// <param name="height">Logical height or native default.</param>
    /// <param name="rowHeight">Logical height per row.</param>
    /// <param name="trackWidth">Track column width.</param>
    /// <param name="ruler">Native ruler label format.</param>
    /// <param name="widthKind">Native width kind.</param>
    /// <param name="width">Native width value.</param>
    /// <param name="flags">Native adapter options.</param>
    /// <returns>The proposed edit or observation.</returns>
    public unsafe CuiTimelineEdit? TimelineEditor(string id, CuiTimelineTrack[] tracks, CuiTimelineRange view, float start, float end, float fps, float height, float rowHeight, float trackWidth, int ruler, int widthKind, float width, int flags)
    {
        CheckFrame();
        CheckExtent(widthKind, width, nameof(width));
        using var memory = new ChartMemory();
        var native = new NativeTimelineTrack[tracks.Length];
        for (int i = 0; i < native.Length; i++)
            native[i] = PackTimelineTrack(tracks[i], memory);

        var layout = new NativeTimelineOptions { Start = start, End = end, Fps = fps, Height = height, RowHeight = rowHeight,
            TrackWidth = trackWidth, Ruler = ruler, WidthKind = widthKind, Width = width, Flags = flags };
        var range = new NativeTimelineView { Time = view.Time, Start = view.Start, End = view.End, Scroll = view.Scroll,
            Track = view.Track, Key = view.Key, Range = view.HasRange ? 1 : 0, In = view.In, Out = view.Out };
        int result;
        fixed (NativeTimelineTrack* data = native)
            result = NativeMethods.TimelineEditor(context, Stable(id), data, native.Length, ref range, in layout);
        if (result == 0) return null;

        var updated = new CuiTimelineTrack[native.Length];
        for (int i = 0; i < updated.Length; i++)
            updated[i] = ReadTimelineTrack(native[i], tracks[i]);
        return new(updated, new(range.Time, range.Start, range.End, range.Scroll, range.Track, range.Key,
            range.Range != 0, range.In, range.Out), (result & 2) != 0);
    }

    private static unsafe NativeTimelineTrack PackTimelineTrack(CuiTimelineTrack track, ChartMemory memory)
    {
        int capacity = Math.Max(track.Capacity, track.Keys.Length);
        var keys = new CuiCurveKey[capacity];
        var selected = new byte[capacity];
        for (int i = 0; i < track.Keys.Length; i++)
        {
            keys[i] = track.Keys[i].Curve;
            selected[i] = track.Keys[i].Selected ? (byte)1 : (byte)0;
        }
        var clips = new NativeTimelineClip[track.Clips.Length];
        for (int i = 0; i < clips.Length; i++)
        {
            var clip = track.Clips[i];
            clips[i] = new() { Start = clip.Start, End = clip.End, Offset = clip.Offset,
                Name = memory.Text(clip.Name), Selected = clip.Selected ? (byte)1 : (byte)0 };
        }
        return new() { Name = memory.Text(track.Name), Icon = memory.Text(OptionalAi(track.Icon)), Depth = track.Depth,
            Collapsed = track.Collapsed ? (byte)1 : (byte)0, Muted = track.Muted ? (byte)1 : (byte)0,
            Locked = track.Locked ? (byte)1 : (byte)0, Color = (byte)track.Color, Keys = memory.Copy<CuiCurveKey>(keys),
            KeyCount = track.Keys.Length, KeyCapacity = capacity, KeySelected = memory.Copy<byte>(selected),
            Clips = memory.Copy<NativeTimelineClip>(clips), ClipCount = clips.Length };
    }

    private static unsafe CuiTimelineTrack ReadTimelineTrack(NativeTimelineTrack native, CuiTimelineTrack track)
    {
        var keys = new CuiTimelineKey[native.KeyCount];
        for (int i = 0; i < keys.Length; i++)
            keys[i] = new(native.Keys[i], native.KeySelected[i] != 0);

        var clips = new CuiTimelineClip[native.ClipCount];
        for (int i = 0; i < clips.Length; i++)
        {
            var clip = native.Clips[i];
            clips[i] = new(clip.Start, clip.End, clip.Offset, Marshal.PtrToStringUTF8((nint)clip.Name) ?? "", clip.Selected != 0);
        }
        return track with { Collapsed = native.Collapsed != 0, Muted = native.Muted != 0, Locked = native.Locked != 0,
            Keys = keys, Clips = clips };
    }
}
