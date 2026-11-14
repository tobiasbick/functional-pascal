#ifndef FPAS_CUI_DOT_DISPLAY_H
#define FPAS_CUI_DOT_DISPLAY_H

CUI_API void fpas_cui_dot_display(cui_ctx *ui, const char *id, const char *text, float pitch,
    int palette, uint32_t rgba, int align, int width_kind, float width, int flags);

#endif
