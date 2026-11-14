#ifndef FPAS_SHEET_H
#define FPAS_SHEET_H

/** Opens a controlled CUI sheet with a stable identity and tracks its native close button.
 * @param ui Current UI-thread context.
 * @param id Stable identity.
 * @param title Visible header.
 * @param description Optional supporting text.
 * @param side CUI edge.
 * @param size Edge-axis extent in logical units.
 * @param open Model-owned visibility.
 * @param enabled Whether the sheet can accept dismissal input.
 * @param focus Whether to focus the close button.
 * @return Bit 1 requests drawing, bit 2 proposes dismissal. */
CUI_API int fpas_cui_sheet_begin(cui_ctx *ui, const char *id, const char *title, const char *description,
                                int side, float size, int open, int enabled, int focus);

/** Closes a native sheet drawing scope and restores the layout target.
 * @param ui Current UI-thread context.
 * @param base Frame-local base view node. */
CUI_API void fpas_cui_sheet_end(cui_ctx *ui, int base);

#endif
