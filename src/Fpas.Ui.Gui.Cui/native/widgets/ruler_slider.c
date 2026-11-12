#include "../fpas_cui.h"

static cui_id fpas_ruler_scroll_key(cui_id track) { return track ^ 0x52554C52u; }

static void fpas_paint_ruler_slider(cui_ctx *ui, cui_node *node, void *data) {
    cui__paint_ruler(ui, node, data);
    /* CUI-009 (docs/future/cui-upstream-bugs.md#cui-009): give the tape the wheel
       event before an enclosing viewport consumes it. */
    cui__paint_wheel_hit(ui, fpas_ruler_scroll_key(node->id), cui__xform_rect(ui, node->rect));
}

static void fpas_paint_focused_ruler_slider(cui_ctx *ui, cui_node *node, void *data) {
    fpas_paint_ruler_slider(ui, node, data);
    cui__draw_focus_ring(ui, node->rect, node->radius, 1.0f);
}

static float fpas_ruler_bound(cui_ctx *ui, cui_id group, int root, cui_node *tape,
    float value, float minimum, float maximum, const char *format) {
    const float bounded = cui__clampf(value, minimum, maximum);
    if (bounded == value) return value;

    /* CUI-011 (docs/future/cui-upstream-bugs.md#cui-011): after native wheel
       snapping, keep the model proposal, value caption and tape at the bound. */
    cui__ruler_state *state = cui_state(ui, group, sizeof *state);
    state->shown.x = bounded;
    state->shown.v = 0;
    if (tape->paint == cui__paint_ruler) ((cui__ruler_paint *)tape->paint_data)->value = bounded;
    const int value_row = ui->nodes[ui->nodes[root].first].next;
    fpas_text(ui, ui->nodes[value_row].first, cui__frame_printf(ui, format, (double)bounded));
    return bounded;
}

int fpas_cui_ruler_slider(cui_ctx *ui, const char *id, const char *label, float *value,
    float minimum, float maximum, float step, int major_every, float spacing, float width,
    const char *format, const char *unit, int enabled, int focus, int *track) {
    const cui_id group = cui_make_id(ui, id), key = group ^ 0x2017u;
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (allowed && focus) ui->focus_next_request = key;
    if (!allowed) {
        cui__ruler_state *state = cui__state_lazy(ui, group, sizeof *state, false);
        if (state) state->dragging = false;
    }

    const cui_ruler_opts options = {
        .step = step, .major_every = major_every, .spacing = spacing,
        .width = width > 0 ? CUI_PX(width) : CUI_FILL,
        .format = cui__frame_strdup(ui, format, -1), .unit = unit
    };
    float next = *value;
    cui_push_disabled(ui, !enabled);
    const bool changed = cui_ruler_slider_ex(ui, cui__frame_printf(ui, "x%s", id),
        &next, minimum, maximum, &options);
    cui_pop_disabled(ui);

    const int root = fpas_find(ui, group);
    fpas_literal(ui, root, label, CUI_C(ui, TEXT_SECONDARY));
    if (!allowed) ui->nodes[root].opacity = CUI__DISABLED;
    *track = fpas_find(ui, key);
    ui->last_id = key;
    fpas_own(ui, key, group, fpas_ruler_scroll_key(key), 0);
    cui_node *tape = &ui->nodes[*track];
    if (allowed && changed && isfinite(next))
        next = fpas_ruler_bound(ui, group, root, tape, next, minimum, maximum, options.format);
    if (tape->paint == cui__paint_focus_after) {
        /* CUI-010 (docs/future/cui-upstream-bugs.md#cui-010): preserve the tape
           painter when the native focus ring replaces it. */
        const cui__ruler_state *state = cui_state(ui, group, sizeof *state);
        cui__ruler_paint *paint = cui__frame_alloc(ui, sizeof *paint);
        *paint = (cui__ruler_paint){
            .value = state->shown.x, .min = minimum, .max = maximum, .step = step, .spacing = spacing,
            .major = major_every, .format = options.format, .tick = cui__control_edge(ui, state->hover),
            .major_c = CUI_C(ui, TEXT_SECONDARY), .label = CUI_C(ui, TEXT_MUTED), .marker = cui__emphasis(ui)
        };
        tape->paint = fpas_paint_focused_ruler_slider;
        tape->paint_data = paint;
    } else if (allowed) {
        tape->paint = fpas_paint_ruler_slider;
    }

    if (!allowed || !changed || !isfinite(next) || next == *value) return 0;
    *value = next;
    return 1;
}
