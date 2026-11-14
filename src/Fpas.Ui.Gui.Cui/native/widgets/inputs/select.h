#ifndef FPAS_SELECT_H
#define FPAS_SELECT_H

/** Draws a native select and its optional searchable popup.
 * @param ui Drawing context.
 * @param id Stable trigger identity.
 * @param items Ordered choice labels.
 * @param icons Optional glyphs in choice order. Empty glyphs are allowed.
 * @param count Number of choices.
 * @param selected Incoming index or minus one, updated on selection.
 * @param label Optional caption.
 * @param placeholder Optional text for an unselected trigger.
 * @param searchable Whether the popup includes a filter.
 * @param enabled Whether the select accepts input.
 * @param focus Whether to request trigger focus.
 * @param nodes Receives the trigger and one node per option, or minus one if hidden.
 * @return Nonzero when the popup is open. */
CUI_API int fpas_cui_select(cui_ctx *ui, const char *id, const char *const *items, const char *const *icons,
    int count, int *selected, const char *label, const char *placeholder,
    int searchable, int enabled, int focus, int *nodes);

#endif
