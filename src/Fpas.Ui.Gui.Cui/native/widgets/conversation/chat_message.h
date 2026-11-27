// [Documentation](../../../../../docs/ui/gui/social-and-media/chat-message.md)
#ifndef FPAS_CUI_CHAT_MESSAGE_H
#define FPAS_CUI_CHAT_MESSAGE_H
/** Draws a chat bubble. Flags 1, 2 and 4 select outgoing, grouped and final bubbles.
 * Bit 8 renders Markdown. Bit 16 enables Markdown interaction and output.
 * The activated and hovered URL pointers remain valid until the frame is released.
 * Offset is -1 when a Markdown task is unchanged. */
CUI_API const char *fpas_cui_chat_message(cui_ctx *ui, const char *id, const char *text, const char *author,
    const char *time, const char *status, const char *avatar, int flags, int enabled,
    int *offset, int *line, int *checked, const char **hovered, int *hover_changed);
#endif
