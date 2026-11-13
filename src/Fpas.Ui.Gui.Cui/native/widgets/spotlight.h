#ifndef FPAS_CUI_SPOTLIGHT_H
#define FPAS_CUI_SPOTLIGHT_H
CUI_API int fpas_cui_spotlight_begin(cui_ctx *ui, const char *id, int width_kind, float width,
    int height_kind, float height, float padding, float gap, int palette, uint32_t rgba, int beam);

CUI_API void fpas_cui_spotlight_end(cui_ctx *ui, int node);
#endif
