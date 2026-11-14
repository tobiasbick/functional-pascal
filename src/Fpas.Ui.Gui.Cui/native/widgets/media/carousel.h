#ifndef FPAS_CUI_CAROUSEL_H
#define FPAS_CUI_CAROUSEL_H
CUI_API int fpas_cui_carousel_begin(cui_ctx *ui, const char *id, int count, int index,
    float height, float gap, float advance, int flags, int focus);
CUI_API void fpas_cui_carousel_slide_begin(cui_ctx *ui);
CUI_API void fpas_cui_carousel_slide_end(cui_ctx *ui);
CUI_API int fpas_cui_carousel_end(cui_ctx *ui, int root, int enabled, int *parts);
#endif
