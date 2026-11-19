// [Documentation](../../../../../docs/ui/gui/text-and-display/canvas.md)
#ifndef FPAS_CUI_CANVAS_H
#define FPAS_CUI_CANVAS_H
typedef struct { int kind; float x, y, a, b, radius; uint32_t color; } fpas_canvas_shape;
typedef void (*fpas_canvas_paint_fn)(void *user, cui_ctx *ui, cui_rect area, cui_id images, int disabled);
CUI_API void fpas_cui_canvas_images_finish(cui_ctx *ui, cui_id key, int count);
/* Wheel deltas are in lines and are consumed only for an enabled wheel handler. */
CUI_API int fpas_cui_drawing_canvas(cui_ctx *ui, const char *id, int width_kind, float width, int height_kind, float height, const fpas_canvas_shape *shapes, int count, int enabled, float *x, float *y, float *w, float *h, int wheel, float *dx, float *dy, fpas_canvas_paint_fn paint, void *user, int pointer);
#endif
