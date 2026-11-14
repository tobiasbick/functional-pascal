#ifndef FPAS_CHIP_H
#define FPAS_CHIP_H

/** Draws a native selection chip.
 * @param ui Current UI-thread context.
 * @param id Stable chip identity.
 * @param label Borrowed literal caption.
 * @param icon Borrowed glyph for the unselected state, or NULL.
 * @param selected Current model-owned selection.
 * @param enabled Whether the chip accepts input.
 * @param focus Whether to request keyboard focus.
 * @return Nonzero when CUI proposes toggling the selection. */
CUI_API int fpas_cui_chip(cui_ctx *ui, const char *id, const char *label, const char *icon,
    int selected, int enabled, int focus);

#endif
