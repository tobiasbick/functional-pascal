#ifndef FPAS_REACTIONS_H
#define FPAS_REACTIONS_H

/** Draws CUI's positional reaction group and returns its proposal.
 * @param ui Current UI context.
 * @param id Stable group identity.
 * @param icons One CUI glyph per item.
 * @param counts Current counts, updated in the caller's frame-local array.
 * @param active Current activation flags, updated in the caller's frame-local array.
 * @param count Positive number of items.
 * @param enabled Whether input is accepted.
 * @param focus Zero or one plus the requested item index.
 * @param nodes Output group followed by the item nodes.
 * @return Changed item index, or -1. */
CUI_API int fpas_cui_reactions(cui_ctx *ui, const char *id, const char *const *icons,
    int *counts, int *active, int count, int enabled, int focus, int *nodes);
#endif
