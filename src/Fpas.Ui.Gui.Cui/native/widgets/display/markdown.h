// [Documentation](../../../../../docs/ui/gui/code/markdown.md)
#ifndef FPAS_CUI_MARKDOWN_H
#define FPAS_CUI_MARKDOWN_H
/** Draws a frame-owned copy of Markdown and reports link/task events.
 * @param ui Current UI-thread frame context.
 * @param id Stable widget identity.
 * @param text NUL-terminated UTF-8 Markdown, copied into frame storage.
 * @param width_kind Extent kind: content zero, fill one, fixed two, percentage three.
 * @param width Logical fixed width or percentage, according to width_kind.
 * @param size Logical text size, or zero for the native default.
 * @param flags Enabled bit one, compact bit two, streaming bit four.
 * @param offset Receives the UTF-8 task byte offset, or minus one.
 * @param line Receives the one-based task line.
 * @param checked Receives the proposed task checked state.
 * @param hovered Receives the hovered URL, or NULL; copy before the frame is released.
 * @param hover_changed Receives whether the hovered URL changed from the previous draw.
 * @return Activated URL, or NULL; copy before the frame is released.
 */
CUI_API const char *fpas_cui_markdown(cui_ctx *ui, const char *id, const char *text, int width_kind, float width, float size, int flags, int *offset, int *line, int *checked, const char **hovered, int *hover_changed);
#endif
