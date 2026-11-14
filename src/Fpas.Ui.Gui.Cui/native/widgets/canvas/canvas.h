#ifndef FPAS_CUI_CANVAS_H
#define FPAS_CUI_CANVAS_H
typedef struct { int kind; float x, y, a, b, radius; uint32_t color; } fpas_canvas_shape;
CUI_API int fpas_cui_drawing_canvas(cui_ctx *ui, const char *id, int width_kind, float width, int height_kind, float height, const fpas_canvas_shape *shapes, int count, int enabled, float *x, float *y, float *w, float *h);
#endif
