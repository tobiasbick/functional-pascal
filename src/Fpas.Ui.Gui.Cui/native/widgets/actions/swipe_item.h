#ifndef FPAS_SWIPE_ITEM_H
#define FPAS_SWIPE_ITEM_H

/** Begins a native swipe row. Child content is drawn before calling fpas_cui_swipe_end.
 * @param ui Current drawing context.
 * @param id Stable row identity.
 * @param labels Ordered left then right action labels.
 * @param icons Optional glyphs for each action.
 * @param tones Native tone ordinal for each action.
 * @param left_count Number of actions on the left.
 * @param right_count Number of actions on the right.
 * @param enabled Whether the row accepts input.
 * @param icon Optional leading glyph for the row.
 * @param nodes Output row and revealed action nodes.
 * @return Native outer node used by fpas_cui_swipe_end. */
CUI_API int fpas_cui_swipe_begin(cui_ctx *ui, const char *id, const char *const *labels,
    const char *const *icons, const int *tones, int left_count, int right_count, int enabled,
    const char *icon, int *nodes);

/** Closes the swipe row and reports a native action index.
 * @param ui Current drawing context.
 * @param node Outer node returned by fpas_cui_swipe_begin.
 * @param detail Optional trailing caption.
 * @return Left index, or right index plus 100, or -1 when no action was chosen. */
CUI_API int fpas_cui_swipe_end(cui_ctx *ui, int node, const char *detail);
#endif
