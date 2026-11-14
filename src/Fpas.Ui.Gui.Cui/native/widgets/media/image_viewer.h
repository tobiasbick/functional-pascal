#ifndef FPAS_CUI_IMAGE_VIEWER_H
#define FPAS_CUI_IMAGE_VIEWER_H
CUI_API int fpas_cui_image_viewer(cui_ctx *ui, const char *id, const fpas_media_source *source,
    cui_image_view *current, int width_kind, float width, int height_kind, float height,
    float maximum, int flags, int focus);
#endif
