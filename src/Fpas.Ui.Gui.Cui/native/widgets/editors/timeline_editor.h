// [Documentation](../../../../../docs/ui/gui/editing-tools/timeline-editor.md)
#ifndef FPAS_CUI_TIMELINE_EDITOR_H
#define FPAS_CUI_TIMELINE_EDITOR_H
/** Detached timeline state, updated with native proposals. */
typedef struct {
    float time, start, end, scroll, in, out; /**< Playhead, visible bounds, vertical scroll and in/out times. */
    int track, key, range; /**< Selected track/key (minus one for none), and whether in/out is enabled. */
} fpas_timeline_view;
/** Borrowed timeline time scale, layout and input settings. */
typedef struct {
    float start, end, fps, height, row_height, track_width, width; /**< Time bounds, frames/second and logical extents. */
    int ruler, width_kind, flags; /**< Native ruler, width extent kind, and enabled/toggles/free-drag/focus bits one/two/four/eight. */
} fpas_timeline_options;
/** Draws timeline tracks and updates their model proposals.
 * @param ui Current UI-thread frame context.
 * @param id Stable widget identity.
 * @param tracks Writable track array; nested key buffers use each track's count and capacity.
 * @param count Number of tracks.
 * @param view Input/output playhead, visible range, scroll, selection and optional in/out range.
 * @param settings Borrowed layout/time settings; flags: enabled one, toggles two, free drag four, focus eight.
 * @return Bit one reports a change; bit two requests playback toggle. Zero when disabled.
 */
CUI_API int fpas_cui_timeline_editor(cui_ctx *ui, const char *id, cui_timeline_track *tracks, int count, fpas_timeline_view *view, const fpas_timeline_options *settings);
#endif
