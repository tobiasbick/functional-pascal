// [Documentation](../../../../../docs/ui/gui/canvas-and-bars/painting.md)
#ifndef FPAS_CUI_PAINTING_H
#define FPAS_CUI_PAINTING_H
CUI_API void fpas_cui_canvas_primitive(cui_ctx *ui, int kind, float x, float y, const float *values, uint32_t color, uint32_t second, const char *text);
CUI_API void fpas_cui_canvas_points(cui_ctx *ui, float x, float y, const cui_vec2 *points, int count, float thickness, uint32_t color, int kind);
CUI_API cui_text_metrics fpas_cui_canvas_measure(cui_ctx *ui, int font, float size, const char *text);
CUI_API void fpas_cui_canvas_image(cui_ctx *ui, cui_id key, int index, const fpas_media_source *source, float x, float y, float width, float height, float radius, uint32_t tint);
CUI_API void fpas_cui_canvas_images_finish(cui_ctx *ui, cui_id key, int count);
#endif
