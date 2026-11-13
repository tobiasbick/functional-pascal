#ifndef FPAS_CUI_TOOL_CALL_H
#define FPAS_CUI_TOOL_CALL_H
CUI_API int fpas_cui_tool_call(cui_ctx *ui, const char *id, const char *name, int status, const char *icon,
    const char *detail, const char *input, const char *output, float seconds, int flags, int *header);
#endif
