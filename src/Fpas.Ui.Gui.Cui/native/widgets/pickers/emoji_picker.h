#ifndef FPAS_EMOJI_PICKER_H
#define FPAS_EMOJI_PICKER_H

/** Draws CUI's emoji picker.
 * @param ui Current drawing context.
 * @param id Stable picker identity.
 * @param recent Null-terminated UTF-8 recent emoji in CUI's format.
 * @param capacity Size of the recent buffer in bytes.
 * @param tone Model-owned skin tone, updated when chosen.
 * @param width Logical width, or zero for CUI's default.
 * @param height Logical grid height, or zero for CUI's default.
 * @param size Logical emoji size, or zero for CUI's default.
 * @param placeholder Optional search placeholder.
 * @param empty_text Optional text for empty results.
 * @param hint Optional preview hint.
 * @param search Whether to show the search field.
 * @param preview Whether to show the preview.
 * @param tone_selector Whether to show skin tone controls.
 * @param enabled Whether input is accepted.
 * @param focus Zero for no request, one for search or two for the grid.
 * @param nodes Optional array of 20 native nodes for inspection.
 * @return Selected UTF-8 text, valid until the next frame, or null. */
CUI_API const char *fpas_cui_emoji_picker(cui_ctx *ui, const char *id, char *recent, size_t capacity,
    int *tone, float width, float height, float size, const char *placeholder, const char *empty_text,
    const char *hint, int search, int preview, int tone_selector, int enabled, int focus, int *nodes);
#endif
