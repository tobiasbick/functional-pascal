// [Documentation](../../../../docs/ui/gui/editing-tools/timeline-editor.md)
using System.Runtime.InteropServices;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Editors;

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
/// <param name="Activity">Changes and pointer state reported by the native editor.</param>
public sealed record CuiTimelineEdit(CuiTimelineTrack[] Tracks, CuiTimelineRange View, bool Play, CuiTimelineActivity Activity);

/// <summary>Native timeline action and pointer information.</summary>
/// <param name="Changed">Whether keys, clips or track flags changed.</param>
/// <param name="TimeChanged">Whether the playhead moved.</param>
/// <param name="Scrubbing">Whether the pointer is dragging the playhead.</param>
/// <param name="SelectionChanged">Whether the key or clip selection changed.</param>
/// <param name="ViewChanged">Whether the viewport moved or zoomed.</param>
/// <param name="KeysMoved">Whether selected keys moved.</param>
/// <param name="AddedTrack">Track where a key was added, or minus one.</param>
/// <param name="AddedKey">Index of the new key, or minus one.</param>
/// <param name="RemovedKeys">Number of removed keys.</param>
/// <param name="RemovedClips">Number of removed clips.</param>
/// <param name="ClipTrack">Track of the edited clip, or minus one.</param>
/// <param name="Clip">Index of the edited clip, or minus one.</param>
/// <param name="Toggled">Track whose flag changed, or minus one.</param>
/// <param name="HoveredTrack">Track of the hovered key, or minus one.</param>
/// <param name="HoveredKey">Index of the hovered key, or minus one.</param>
public sealed record CuiTimelineActivity(bool Changed, bool TimeChanged, bool Scrubbing, bool SelectionChanged,
    bool ViewChanged, bool KeysMoved, int AddedTrack, int AddedKey, int RemovedKeys, int RemovedClips,
    int ClipTrack, int Clip, int Toggled, int HoveredTrack, int HoveredKey);

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
    internal int Activity, AddedTrack, AddedKey, RemovedKeys, RemovedClips;
    internal int ClipTrack, Clip, Toggled, HoveredTrack, HoveredKey;
}

[StructLayout(LayoutKind.Sequential)]
internal struct NativeTimelineOptions
{
    internal float Start, End, Fps, Height, RowHeight, TrackWidth, Width;
    internal int Ruler, WidthKind, Flags;
}
