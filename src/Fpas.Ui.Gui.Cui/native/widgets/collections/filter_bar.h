// [Documentation](../../../../../docs/ui/gui/data-grid/filter-bar.md)
#ifndef FPAS_CUI_FILTER_BAR_H
#define FPAS_CUI_FILTER_BAR_H

/** Draws CUI's filter bar with optional search and per-field glyphs.
 * @param ui Current CUI context.
 * @param id Stable bar identity.
 * @param names Field names.
 * @param icons Optional glyphs, one per field. A null entry omits the glyph.
 * @param values All field option captions in field order.
 * @param counts Number of values per field.
 * @param count Number of fields, at most 32.
 * @param selected Current option masks, updated on a proposal.
 * @param search UTF-8 search buffer, or null to omit the search control.
 * @param capacity Buffer size including the terminator.
 * @param enabled Whether the bar accepts input.
 * @param focus Whether to focus the first available preferred control.
 * @param parts Receives the root, search, add, clear and chip nodes.
 * @return Whether a model value changed. */
CUI_API int fpas_cui_filter_bar(cui_ctx *ui, const char *id, const char *const *names,
    const char *const *icons, const char *const *values, const int *counts, int count,
    uint32_t *selected, char *search, int capacity, int enabled, int focus, int *parts);
#endif
