#ifndef FPAS_SEARCH_FIELD_H
#define FPAS_SEARCH_FIELD_H

/** Draws CUI's native search field with a caller-owned UTF-8 buffer.
 * @param ui Drawing context.
 * @param id Stable native identity.
 * @param value NUL-terminated value edited in place.
 * @param capacity Total writable bytes including the terminator.
 * @param label Optional caption.
 * @param placeholder Optional empty-field text.
 * @param icon Optional leading glyph.
 * @param hint Optional helper text.
 * @param error Optional validation error.
 * @param prefix Optional text before the value.
 * @param suffix Optional text after the value.
 * @param size CUI control size.
 * @param read_only Allow selection while preventing edits.
 * @param enabled Whether the editor accepts input.
 * @param focus Request editor focus.
 * @param nodes Receives the editor and affordance node indices.
 * @return CUI_CHANGED and CUI_SUBMITTED flags. */
CUI_API int fpas_cui_search_field(cui_ctx *ui, const char *id, char *value, size_t capacity,
    const char *label, const char *placeholder, const char *icon, const char *hint, const char *error,
    const char *prefix, const char *suffix, int size, int read_only, int enabled, int focus, int *nodes);

#endif
