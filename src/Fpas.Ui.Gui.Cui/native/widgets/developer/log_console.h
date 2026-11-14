#ifndef FPAS_CUI_LOG_CONSOLE_H
#define FPAS_CUI_LOG_CONSOLE_H
typedef struct { const char *text; double time; int level; } fpas_log_entry;
CUI_API int fpas_cui_log_console(cui_ctx *ui, const char *id, const fpas_log_entry *entries, int count, int width_kind, float width, float height, float size, int flags);
#endif
