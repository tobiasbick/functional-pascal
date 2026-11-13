#ifndef FPAS_CUI_REASONING_H
#define FPAS_CUI_REASONING_H
CUI_API int fpas_cui_reasoning_begin(cui_ctx *ui, const char *id, int thinking, float seconds, const char *title, int flags, int *nodes);
CUI_API void fpas_cui_reasoning_end(cui_ctx *ui, int root, int head);
#endif
