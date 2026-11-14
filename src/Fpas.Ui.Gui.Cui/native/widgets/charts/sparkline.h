#ifndef FPAS_CUI_SPARKLINE_H
#define FPAS_CUI_SPARKLINE_H
CUI_API void fpas_cui_sparkline(cui_ctx *ui, const char *id, const float *values, int count,
    int width_kind, float width, float height, int palette, uint32_t rgba);
#endif
