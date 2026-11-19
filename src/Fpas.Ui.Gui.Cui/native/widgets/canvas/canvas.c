// [Documentation](../../../../../docs/ui/gui/text-and-display/canvas.md)
#include "../../fpas_cui.h"

typedef struct {
    int count; const fpas_canvas_shape *shapes; cui_id wheel, images;
    fpas_canvas_paint_fn paint; void *user;
} fpas_canvas_drawing;
typedef struct { bool hovered, pressed; float x, y; } fpas_canvas_pointer;

static void fpas_canvas_paint(cui_ctx *ui, cui_rect area, void *data) {
    const fpas_canvas_drawing *drawing = data;
    if (drawing->wheel) cui_wheel_region(ui, drawing->wheel, area);
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
    if (drawing->paint) drawing->paint(drawing->user, ui, area, drawing->images, cui_is_disabled(ui));
}

int fpas_cui_drawing_canvas(cui_ctx *ui, const char *id, int width_kind, float width, int height_kind, float height, const fpas_canvas_shape *shapes, int count, int enabled, float *x, float *y, float *w, float *h, int wheel, float *dx, float *dy, fpas_canvas_paint_fn paint, void *user, int pointer) {
    fpas_canvas_shape *copy = NULL;
    if (count > 0) {
        const size_t size = (size_t)count * sizeof *shapes;
        copy = cui__frame_alloc(ui, size);
        memcpy(copy, shapes, size);
    }
    const cui_id key = cui_make_id(ui, id);
    cui_push_id(ui, id);
    const cui_id wheel_key = cui_make_id(ui, "wheel");
    const cui_id images_key = cui_make_id(ui, "paint-images");
    cui_pop_id(ui);
    fpas_own(ui, key, wheel ? wheel_key : 0, paint ? images_key : 0, 0);
    cui_push_disabled(ui, !enabled);
    const bool active = enabled && !cui_is_disabled(ui);
    const fpas_canvas_drawing drawing = {count, copy, wheel && active ? wheel_key : 0, images_key, paint, user};
    cui_vec2 delta = {0};
    if (wheel) cui_wheel_take(ui, wheel_key, &delta);
    const cui_canvas_result result = cui_canvas(ui, id, fpas_extent(width_kind, width),
        fpas_extent(height_kind, height), fpas_canvas_paint, &drawing, sizeof drawing);
    cui_pop_disabled(ui);
    *x = result.mouse.x;
    *y = result.mouse.y;
    *w = result.rect.w;
    *h = result.rect.h;
    *dx = delta.x;
    *dy = delta.y;
    int flags = active && result.clicked ? 1 : 0;
    if (pointer) {
        fpas_canvas_pointer *previous = cui_state(ui, key, sizeof *previous);
        const bool hovered = active && result.hovered, pressed = active && result.pressed;
        if (previous->hovered != hovered || previous->pressed != pressed ||
            ((hovered || pressed) && (previous->x != *x || previous->y != *y))) flags |= 2;
        if (hovered) flags |= 4;
        if (pressed) flags |= 8;
        *previous = (fpas_canvas_pointer){hovered, pressed, *x, *y};
    } else {
        fpas_canvas_pointer *previous = cui__state_lazy(ui, key, 0, false);
        if (previous) *previous = (fpas_canvas_pointer){0};
    }
    return flags;
}
