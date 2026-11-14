#ifndef FPAS_CUI_TERMINAL_H
#define FPAS_CUI_TERMINAL_H
CUI_API const char *fpas_cui_terminal(cui_ctx *ui, const char *id, const char *output, int revision, const char *prompt, int width_kind, float width, float height, float size, int flags, int *interrupt);
#endif
