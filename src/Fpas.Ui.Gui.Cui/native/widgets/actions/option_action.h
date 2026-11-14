#ifndef FPAS_CUI_OPTION_ACTION_H
#define FPAS_CUI_OPTION_ACTION_H
CUI_API int fpas_cui_option_action(cui_ctx *ui, const char *id,
    const char *label, const char *const *items, int count, int *value, const char *icon, const char *tooltip, const char *display, int flags, int focus, int *parts);
#endif
