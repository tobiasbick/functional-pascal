// [Documentation](../../../../../docs/ui/gui/app-chrome/split-view.md)
#ifndef FPAS_CUI_SPLIT_VIEW_H
#define FPAS_CUI_SPLIT_VIEW_H

/* All operations use ui, the current UI-thread context. id is a stable retained identity.
 * Returned node indices are valid only in the frame that produced them. */

/** Begins the first pane of a split; pair with next then end in this frame.
 * ratio is the model split proportion; returns its native proposal or -1 at the nesting limit.
 * vertical selects orientation;
 * first/second are minimum pane sizes. Width/height use shared extent kind/value pairs.
 * enabled gates splitter input; content admission remains the caller's responsibility. */
CUI_API float fpas_cui_split_view_begin(cui_ctx *ui, const char *id, float ratio, int vertical,
    float first, float second, int width_kind, float width, int height_kind, float height, int enabled);
/** Ends the first pane and begins the second; enabled controls handle interactivity.
 * Returns 1 on success, or 0 if the first pane is not active. */
CUI_API int fpas_cui_split_view_next(cui_ctx *ui, int enabled);
/** Closes the matching split and makes its container the last layout target.
 * Returns 1 on success, or 0 if the second pane is not active. */
CUI_API int fpas_cui_split_view_end(cui_ctx *ui);

#endif
