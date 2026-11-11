#ifndef FPAS_TABS_H
#define FPAS_TABS_H

/** Draws CUI's tab strip with a controlled selection.
 * @param ui Current UI-thread context.
 * @param id Stable strip identity.
 * @param labels Borrowed literal captions, one per tab.
 * @param icons Borrowed glyphs, one per tab, with empty strings for no icon.
 * @param count Number of tabs, one through 32.
 * @param selected Current zero-based selection.
 * @param variant Native underline or pill variant.
 * @param full_width Whether to distribute tabs across the available width.
 * @param enabled Whether the strip accepts input.
 * @param focus Zero or one plus the requested tab index.
 * @param nodes Receives the strip and tab node indices.
 * @return Proposed tab index, or minus one when unchanged. */
CUI_API int fpas_cui_tabs(cui_ctx *ui, const char *id, const char *const *labels,
    const char *const *icons, int count, int selected, int variant, int full_width,
    int enabled, int focus, int *nodes);

#endif
