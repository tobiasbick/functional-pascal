// [Documentation](../../../../../docs/ui/gui/text-and-display/canvas.md)
#include "../../fpas_cui.h"

typedef struct {
    int count; const fpas_canvas_shape *shapes; cui_id wheel, images, id;
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
    if (drawing->paint) {
        int state = (cui_is_disabled(ui) ? FPAS_PAINT_DISABLED : 0)
            | (cui_focused_id(ui) == drawing->id ? FPAS_PAINT_FOCUSED : 0)
            | (cui_focus_visible(ui, drawing->id) ? FPAS_PAINT_FOCUS_VISIBLE : 0);
        drawing->paint(drawing->user, ui, area, drawing->images, state);
    }
}

int fpas_cui_drawing_canvas(cui_ctx *ui, const char *id, int width_kind, float width, int height_kind, float height, const fpas_canvas_shape *shapes, int count, int enabled, float *x, float *y, float *w, float *h, int wheel, float *dx, float *dy, fpas_canvas_paint_fn paint, void *user, int pointer, int focusable, int component_input, int focus, int cursor) {
    fpas_canvas_shape *copy = NULL;
    if (count > 0) {
        const size_t size = (size_t)count * sizeof *shapes;
        copy = cui_frame_alloc(ui, size);
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
    const fpas_canvas_drawing drawing = {count, copy, wheel && active ? wheel_key : 0, images_key, key, paint, user};
    cui_vec2 delta = {0};
    if (wheel) cui_wheel_take(ui, wheel_key, &delta);
    const cui_canvas_opts options = { .focusable = focusable || component_input, .component_input = component_input != 0 };
    if (active && focus && options.focusable) {
        if (component_input) cui_request_component_focus(ui, key);
        else fpas_cui_request_focus(ui, id);
    }
    const cui_canvas_result result = cui_canvas_ex(ui, id, fpas_extent(width_kind, width),
        fpas_extent(height_kind, height), fpas_canvas_paint, &drawing, sizeof drawing, &options);
    if (active && result.hovered && cursor >= 0) cui_set_cursor(ui, (cui_cursor)cursor);
    cui_pop_disabled(ui);
    *x = result.mouse.x;
    *y = result.mouse.y;
    *w = result.rect.w;
    *h = result.rect.h;
    *dx = delta.x;
    *dy = delta.y;
    int flags = active && result.clicked ? 1 : 0;
    if (active && component_input) flags |= 16;
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

typedef struct { cui_id target; int repeated; } fpas_canvas_route;

static bool fpas_canvas_route_event(cui_ctx *ui, void *user, const cui_input_event *event) {
    fpas_canvas_route *route = user;
    cui_id id = cui_focused_id(ui);
    if (event->kind == CUI_INPUT_KEY && event->key == CUI_KEY_TAB) return false;
    if (cui_component_has_input(ui, id)) route->target = id;
    route->repeated = cui_input_repeated(ui);
    return route->target != 0;
}

CUI_API cui_id fpas_cui_canvas_dispatch(cui_ctx *ui, int kind, int key, int mods, int down, const char *text, int *repeated) {
    const cui_input_event event = { .kind = (cui_input_event_kind)kind, .key = (cui_key)key,
        .mods = mods, .down = down != 0, .focused = down != 0, .text = text };
    fpas_canvas_route route = {0};
    /* A blur revokes eligibility during dispatch. Record its previous recipient. */
    if (kind == CUI_INPUT_FOCUS && !down && cui_component_has_input(ui, cui_focused_id(ui)))
        route.target = cui_focused_id(ui);
    cui_dispatch_input(ui, &event, fpas_canvas_route_event, &route);
    *repeated = route.repeated;
    return route.target;
}
