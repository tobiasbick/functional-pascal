#ifndef FPAS_TOGGLE_GROUP_H
#define FPAS_TOGGLE_GROUP_H

/** Draws CUI's native toggle group with model-owned selection.
 * @param ui Current UI-thread context.
 * @param id Stable group identity.
 * @param labels Borrowed literal captions.
 * @param icons Borrowed glyphs, empty entries or NULL for text items.
 * @param count Number of items, from one to 32.
 * @param selected Model-owned selection bits in item order.
 * @param size CUI control size.
 * @param exclusive Whether exactly one item is selected.
 * @param icon_only Whether items with glyphs use their labels as tooltips.
 * @param enabled Whether the items receive input and focus.
 * @param focus Zero for none, or one plus the requested focus ordinal.
 * @param nodes Receives the group followed by its ordered item nodes.
 * @return The selection bits proposed by CUI. */
CUI_API uint32_t fpas_cui_toggle_group(cui_ctx *ui, const char *id, const char *const *labels,
    const char *const *icons, int count, uint32_t selected, int size, int exclusive,
    int icon_only, int enabled, int focus, int *nodes);

#endif
