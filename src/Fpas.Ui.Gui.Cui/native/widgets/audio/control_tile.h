#ifndef FPAS_CUI_CONTROL_TILE_H
#define FPAS_CUI_CONTROL_TILE_H
CUI_API int fpas_cui_control_tile(cui_ctx *ui, const char *id, const char *label, const char *icon,
    const char *detail, int *on, int width_kind, float width, int palette, uint32_t rgba, int flags);
#endif
