#include "../fpas_cui.h"

typedef struct { int count; const fpas_canvas_shape *shapes; } fpas_canvas_drawing;

static void fpas_canvas_paint(cui_ctx *ui, cui_rect area, void *data) {
    const fpas_canvas_drawing *drawing = data;
    for (int i = 0; i < drawing->count; ++i) {
        const fpas_canvas_shape *shape = &drawing->shapes[i];
        const cui_color color = fpas_chart_color(-1, shape->color);
        const cui_vec2 point = { area.x + shape->x, area.y + shape->y };
        if (shape->kind == 0)
            cui_draw_rect(ui, (cui_rect){point.x, point.y, shape->a, shape->b}, CUI_CORNERS(shape->radius), color);
        else if (shape->kind == 1)
            cui_draw_circle(ui, point, shape->radius, color);
        else
            cui_draw_line(ui, point, (cui_vec2){area.x + shape->a, area.y + shape->b}, shape->radius, color);
    }
}

int fpas_cui_drawing_canvas(cui_ctx *ui, const char *id, int width_kind, float width, int height_kind, float height, const fpas_canvas_shape *shapes, int count, int enabled, float *x, float *y, float *w, float *h) {
    fpas_canvas_shape *copy = NULL;
    if (count > 0) {
        const size_t size = (size_t)count * sizeof *shapes;
        copy = cui__frame_alloc(ui, size);
        memcpy(copy, shapes, size);
    }
    const fpas_canvas_drawing drawing = {count, copy};
    cui_push_disabled(ui, !enabled);
    const cui_canvas_result result = cui_canvas(ui, id, fpas_extent(width_kind, width),
        fpas_extent(height_kind, height), fpas_canvas_paint, &drawing, sizeof drawing);
    cui_pop_disabled(ui);
    *x = result.mouse.x;
    *y = result.mouse.y;
    *w = result.rect.w;
    *h = result.rect.h;
    return enabled && result.clicked;
}
