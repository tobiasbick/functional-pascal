// [Documentation](../../../../../docs/ui/gui/text-and-display/canvas.md)
#ifndef FPAS_CUI_CANVAS_H
#define FPAS_CUI_CANVAS_H
typedef struct { int kind; float x, y, a, b, radius; uint32_t color; } fpas_canvas_shape;
enum { FPAS_PAINT_DISABLED = 1, FPAS_PAINT_FOCUSED = 2, FPAS_PAINT_FOCUS_VISIBLE = 4 };
CUI_API cui_id fpas_cui_canvas_dispatch(cui_ctx *ui, int kind, int key, int mods, int down, const char *text, int *repeated);
/* Returns nonzero when the hit defers a focused canvas's focus decision. */
CUI_API int fpas_cui_canvas_pointer_focus(cui_ctx *ui, float x, float y);
/* A nonzero return replaces the default whole-canvas wheel region. */
typedef int (*fpas_canvas_paint_fn)(void *user, cui_ctx *ui, cui_rect area, cui_id images, int state, cui_id wheel);
CUI_API void fpas_cui_canvas_images_finish(cui_ctx *ui, cui_id key, int count);
/* Wheel deltas are in lines. Enabled=2 preserves eligibility during a passive view.
   Result bit 32 reports the native right-button press on the hovered canvas. */
CUI_API int fpas_cui_drawing_canvas(cui_ctx *ui, const char *id, int width_kind, float width, int height_kind, float height, const fpas_canvas_shape *shapes, int count, int enabled, float *x, float *y, float *w, float *h, int wheel, float *dx, float *dy, fpas_canvas_paint_fn paint, void *user, int pointer, int focusable, int component_input, int focus, int cursor);
#endif
