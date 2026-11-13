#ifndef FPAS_CUI_PALETTE_STRIP_H
#define FPAS_CUI_PALETTE_STRIP_H
CUI_API int fpas_cui_palette_strip(cui_ctx *ui, const char *id,
    const char *label, uint32_t *colors, int *locked, int count, uint32_t *seed, const char *display, float height, int flags, int focus, int *parts);
#endif
