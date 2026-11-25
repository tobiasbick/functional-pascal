// [Documentation](../../../../../docs/ui/gui/text-and-display/canvas.md)
#include "../../fpas_cui.h"

typedef struct {
    int count; const fpas_canvas_shape *shapes; cui_id wheel, images, id;
    fpas_canvas_paint_fn paint; void *user;
} fpas_canvas_drawing;
typedef struct { bool hovered, pressed; float x, y; } fpas_canvas_pointer;
typedef struct { uint64_t frame; cui_id target; } fpas_canvas_press;

static cui_id fpas_canvas_press_key(void) { return cui__hash_str("#fpas_canvas_press", 0); }

static const cui__focus_entry *fpas_previous_focus_entry(cui_ctx *ui, cui_id key) {
    for (int i = 0; i < ui->prev_focus_count; ++i)
        if (ui->prev_focus_order[i].id == key) return &ui->prev_focus_order[i];
    return NULL;
}

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
    /* Passive resize drawing keeps the preceding frame's focus eligibility. */
    if (enabled == 2) enabled = fpas_previous_focus_entry(ui, key) != NULL;
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
    /* A replayed press already changed focus. Keep any subsequent Tab move
       when the canvas handles the same native press during submission. */
    const fpas_canvas_press *press = ui->mpressed[CUI_MOUSE_LEFT]
        ? cui__state_lazy(ui, fpas_canvas_press_key(), 0, false) : NULL;
    const bool keep_focus = press && press->frame == ui->frame && press->target == key &&
        ui->focus != key && ui->focus_next_request != key;
    const cui_id before_focus = ui->focus, before_owner = ui->component_input.focus_owner;
    const bool before_visible = ui->focus_visible;
    const cui_canvas_result result = cui_canvas_ex(ui, id, fpas_extent(width_kind, width),
        fpas_extent(height_kind, height), fpas_canvas_paint, &drawing, sizeof drawing, &options);
    if (keep_focus && ui->focus == key) {
        ui->focus = before_focus;
        ui->component_input.focus_owner = before_owner;
        ui->focus_visible = before_visible;
    }
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

/* Resolve the recorded press position against the previous layout, using
   CUI's topmost hit and occlusion flags. The normal pointer path owns clicks. */
CUI_API int fpas_cui_canvas_pointer_focus(cui_ctx *ui, float x, float y) {
    cui_id target = 0;
    for (int i = ui->prev_hit_count - 1; i >= 0; --i) {
        const cui_hit *hit = &ui->prev_hits[i];
        if (!cui__rect_contains(hit->rect, x, y)) continue;
        if (hit->flags & CUI_NF_INTERACTIVE) { target = hit->id; break; }
        if (hit->flags & (CUI_NF_BLOCKER | CUI_NF_HIT_OPAQUE)) break;
    }
    if (!target || (ui->active && ui->active != target)) return 0;
    const cui__focus_entry *entry = fpas_previous_focus_entry(ui, target);
    if (!entry) return ui->focus && ui->focus == ui->component_input.focus_owner;
    if (!entry->component_input && !ui->component_input.focus_owner) return 0;
    ui->focus = target;
    ui->component_input.focus_owner = entry->component_input ? target : 0;
    ui->focus_visible = false;
    fpas_canvas_press *press = cui_state(ui, fpas_canvas_press_key(), sizeof *press);
    *press = (fpas_canvas_press){ui->frame, target};
    return 0;
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
    const int previous_mods = cui_key_mods(ui);
    /* A blur revokes eligibility during dispatch. Record its previous recipient. */
    if (kind == CUI_INPUT_FOCUS && !down && cui_component_has_input(ui, cui_focused_id(ui)))
        route.target = cui_focused_id(ui);
    cui_dispatch_input(ui, &event, fpas_canvas_route_event, &route);
    /* Unhandled releases keep the same modifier state as cui_input_key. */
    if (kind == CUI_INPUT_KEY && key != CUI_KEY_NONE && !down && !route.target)
        ui->mods = previous_mods;
    *repeated = route.repeated;
    return route.target;
}
