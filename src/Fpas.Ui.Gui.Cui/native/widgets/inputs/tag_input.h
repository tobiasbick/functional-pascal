#ifndef FPAS_TAG_INPUT_H
#define FPAS_TAG_INPUT_H

/** Draws a native tag input and returns the nodes for its editor and remove buttons.
 * @param ui Drawing context.
 * @param id Stable widget identity.
 * @param value Writable comma-separated UTF-8 tag buffer.
 * @param capacity Size of value including the terminator.
 * @param label Optional caption.
 * @param placeholder Optional empty-field text.
 * @param max_tags Maximum number of tags.
 * @param enabled Whether the field accepts input.
 * @param focus Whether to focus the draft editor.
 * @param count Number of tags supplied in value.
 * @param nodes Receives the editor node followed by count remove-button nodes. */
CUI_API void fpas_cui_tag_input(cui_ctx *ui, const char *id, char *value, size_t capacity,
    const char *label, const char *placeholder, int max_tags, int enabled, int focus, int count, int *nodes);

#endif
