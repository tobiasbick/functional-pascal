#ifndef FPAS_CUI_IMAGE_CROPPER_H
#define FPAS_CUI_IMAGE_CROPPER_H
CUI_API int fpas_cui_image_cropper(cui_ctx *ui, const char *id, const fpas_media_source *source,
    cui_rect *current, int width_kind, float width, int height_kind, float height,
    float aspect, float minimum, int flags, int focus);
#endif
