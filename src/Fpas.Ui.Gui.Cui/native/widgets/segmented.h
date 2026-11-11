#ifndef FPAS_SEGMENTED_H
#define FPAS_SEGMENTED_H

/** Draws CUI's native segmented primitive without changing the host palette.
 * @param ui Current UI-thread context.
 * @param id Stable group identity.
 * @param labels Native CUI labels; empty text produces an icon-only segment.
 * @param icons Borrowed glyphs, or NULL for text segments.
 * @param count Number of segments, from one to 32.
 * @param selected Model-owned ordinal in the available range.
 * @param size CUI control size.
 * @param full_width Whether segments fill their allocation.
 * @param enabled Whether segments receive input and focus.
 * @param focus Zero for none, or one plus the requested focus ordinal.
 * @param nodes Receives the group followed by its ordered segment nodes.
 * @return Proposed ordinal on a change, otherwise -1. */
CUI_API int fpas_cui_segmented(cui_ctx *ui, const char *id, const char *const *labels,
    const char *const *icons, int count, int selected, int size, int full_width,
    int enabled, int focus, int *nodes);

#endif
