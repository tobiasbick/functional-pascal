#ifndef FPAS_CUI_IMAGE_COMPARE_H
#define FPAS_CUI_IMAGE_COMPARE_H
CUI_API int fpas_cui_image_compare(cui_ctx *ui, const char *id, const fpas_media_source *before,
    const fpas_media_source *after, float *position, int width_kind, float width, float height,
    float aspect, const char *before_label, const char *after_label, int enabled, int focus);
#endif
