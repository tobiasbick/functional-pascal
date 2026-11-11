#ifndef FPAS_MENU_H
#define FPAS_MENU_H

/** Draws a native menu with literal captions and stable keyed commands.
 * @param ui Current UI-thread context.
 * @param id Stable trigger identity.
 * @param label Trigger caption and tooltip.
 * @param icon Optional trigger glyph.
 * @param keys Stable command identities.
 * @param labels Literal row captions.
 * @param icons Row glyphs, empty when absent.
 * @param shortcuts Display-only row hints, empty when absent.
 * @param flags Row kind (action=0, label=1, separator=2), disabled=4, danger=8, checked=16.
 * @param count Number of rows.
 * @param variant Shared button variant ordinal.
 * @param placement Native placement ordinal.
 * @param width Logical popup width, zero for content sizing.
 * @param icon_only Use a square icon trigger.
 * @param enabled Whether input is permitted.
 * @param focus Request trigger focus.
 * @param context_menu Attach the menu to the previous component instead of drawing a trigger.
 * @param nodes Output trigger/target and command nodes, or -1.
 * @param open Whether the popup accepts input.
 * @param highlighted Highlighted entry index, or -1.
 * @param owner Native lifecycle key for context-popup retirement.
 * @return Chosen entry index, or -1. */
CUI_API int fpas_cui_menu(cui_ctx *ui, const char *id, const char *label, const char *icon,
    const char *const *keys, const char *const *labels, const char *const *icons,
    const char *const *shortcuts, const int *flags, int count, int variant, int placement,
    float width, int icon_only, int enabled, int focus, int context_menu, int *nodes,
    int *open, int *highlighted, unsigned int *owner);
#endif
