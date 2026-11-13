#ifndef FPAS_CUI_CHAT_MESSAGE_H
#define FPAS_CUI_CHAT_MESSAGE_H
CUI_API void fpas_cui_chat_message(cui_ctx *ui, const char *id, const char *text, const char *author,
    const char *time, const char *status, const char *avatar, int flags, int enabled);
#endif
