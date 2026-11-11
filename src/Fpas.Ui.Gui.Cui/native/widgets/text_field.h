#ifndef FPAS_TEXT_FIELD_H
#define FPAS_TEXT_FIELD_H

/** Draws CUI's single-line editor using a caller-owned UTF-8 buffer.
 * @param ui Owner drawing context.
 * @param id Stable native identity.
 * @param value NUL-terminated value; edited in place.
 * @param capacity Total writable bytes, including the terminator.
 * @param label Optional caption.
 * @param placeholder Optional empty-field text.
 * @param icon Optional leading glyph.
 * @param hint Optional helper text.
 * @param error Optional validation error.
 * @param prefix Optional text before the value.
 * @param suffix Optional text after the value.
 * @param size CUI control size.
 * @param password Mask the value with a native reveal button.
 * @param clearable Show the native clear button when editable and nonempty.
 * @param read_only Allow selection while preventing edits.
 * @param enabled Whether the field and its actions accept input.
 * @param focus Request editor focus.
 * @param nodes Receives the editor, clear and reveal node indices or -1.
 * @param revealed Receives whether a password is displayed in plain text.
 * @return CUI_CHANGED and CUI_SUBMITTED flags. */
CUI_API int fpas_cui_text_field(cui_ctx *ui, const char *id, char *value, size_t capacity,
    const char *label, const char *placeholder, const char *icon, const char *hint, const char *error,
    const char *prefix, const char *suffix, int size, int password, int clearable,
    int read_only, int enabled, int focus, int *nodes, int *revealed);

#endif
