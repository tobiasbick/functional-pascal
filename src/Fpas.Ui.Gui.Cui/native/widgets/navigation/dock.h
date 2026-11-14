#ifndef FPAS_CUI_DOCK_H
#define FPAS_CUI_DOCK_H
CUI_API int fpas_cui_dock(cui_ctx *ui, const char *id, const char *const *icons, const char *const *labels,
    const int *palettes, const uint32_t *colors, const int *badges, const int *kinds, int count,
    float size, float magnify, int enabled, int focus, int *parts);
#endif
