// [Documentation](../../../../../docs/ui/gui/creative-controls/ratio-slider.md)
#include "../../fpas_cui.h"

/** Reserves enough space on each side for the literal caption and percentage. */
static void fpas_ratio_limits(cui_ctx *ui, const char *first, const char *second, float width,
    int first_length, int second_length, float *lower, float *upper, bool *names) {
    if (cui__display_len(first) == first_length && cui__display_len(second) == second_length) {
        cui__ratio_limits(ui, first, second, width, lower, upper, names);
        return;
    }
    const float fs = cui__fs(ui, ui->theme.text_lg), available = width - CUI__RATIO_HANDLE;
    if (available <= 0) {
        *lower = 0.05f;
        *upper = 0.95f;
        *names = true;
        return;
    }
    const float percentage = 2 * CUI__RATIO_PAD + cui__text_width(ui, CUI_FONT_MONO, fs, "100%", 4, true);
    const float a = percentage + CUI__RATIO_GAP + cui__text_width(ui, CUI_FONT_MONO, fs, first, first_length, false);
    const float b = percentage + CUI__RATIO_GAP + cui__text_width(ui, CUI_FONT_MONO, fs, second, second_length, false);
    *names = a + b <= available;
    *lower = (*names ? a : percentage) / available;
    *upper = 1 - (*names ? b : percentage) / available;
    if (*lower > *upper) *lower = *upper = 0.5f;
}

int fpas_cui_ratio_slider(cui_ctx *ui, const char *id,
    const char *first, const char *second, float *value, int tone, float step, float height, int width_kind, float width, int flags, int focus, int *parts) {
    // CUI-158 (docs/future/cui-upstream-bugs.md#cui-158): use the stable ID for input and paint state.
    const cui_id key = cui_make_id(ui, id);
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    if (enabled && focus == 0) ui->focus_next_request = key;

    // CUI-100 (docs/future/cui-upstream-bugs.md#cui-100): apply the recorded release position.
    const fpas_slider_release release = fpas_slider_release_begin(ui, key, enabled);
    cui_push_disabled(ui, !enabled);
    const cui_interact input = cui__interact(ui, key, CUI_IF_FOCUSABLE | CUI_IF_DRAG);
    cui_pop_disabled(ui);
    const cui_rect previous = cui__prev_rect(ui, key);
    cui__ratio_state *state = cui_state(ui, key, sizeof *state);
    const int first_length = (int)strlen(first), second_length = (int)strlen(second);
    float lower, upper;
    bool names;
    fpas_ratio_limits(ui, first, second, previous.w, first_length, second_length, &lower, &upper, &names);
    const float old = *value;
    const float current = cui__clampf(old, 0, 1);
    step = step > 0 ? step : .01f;
    if (!state->init) {
        state->t.x = cui__clampf(current, lower, upper);
        state->init = true;
    }
    bool jumped = false;
    if (input.active && ui->mdown[CUI_MOUSE_LEFT] && previous.w > CUI__RATIO_HANDLE) {
        const float x = cui__mirror_x(previous, state->rtl, ui->mx);
        const float target = (x - previous.x - CUI__RATIO_HANDLE * .5f) / (previous.w - CUI__RATIO_HANDLE);
        if (ui->mpressed[CUI_MOUSE_LEFT]) {
            state->dragging = true;
            jumped = fabsf(target - current) > .03f;
        }
        *value = cui__clampf(roundf(target / step) * step, lower, upper);
    } else {
        state->dragging = false;
        const float target = input.keyboard_focused ? cui__slider_keys(ui, current, step, state->rtl) : current;
        if (target != current) *value = cui__clampf(roundf(target / step) * step, lower, upper);
    }
    fpas_slider_release_end(ui, release);
    const float shown = cui__clampf(*value, lower, upper);
    if (jumped || !state->dragging) cui__spring(ui, &state->t, shown, CUI_SPRING_SMOOTH);
    else state->t = (cui_spring){ .x = shown };
    cui__tween(ui, &state->hover, input.hovered || input.active, CUI__T_HOVER);
    cui__tween(ui, &state->focus, input.focus_visible, CUI__T_FOCUS);
    if (input.hovered || input.active) cui__set_cursor(ui, CUI_CURSOR_RESIZE_EW);

    const cui_size extent = fpas_extent(width_kind, width);
    const int node = cui__leaf(ui, key, extent.kind ? extent : CUI_FILL, CUI_PX(height > 0 ? height : 52));
    cui_node *native = &ui->nodes[node];
    native->self_align = CUI_ALIGN_STRETCH;
    native->opacity = enabled ? 1.0f : CUI__DISABLED;
    cui__mark_interactive(ui, node, CUI_IF_FOCUSABLE);
    const cui_color color = cui__tone_color(ui, tone == CUI_TONE_NEUTRAL ? CUI_TONE_ACCENT : tone);
    const cui_color under = cui_mix(CUI_C(ui, BACKGROUND), color, .18f);
    const cui_color track = CUI_C(ui, CONTROL_TRACK);
    cui__ratio_paint *paint = CUI_PAINT_DATA(ui, cui__ratio_paint);
    *paint = (cui__ratio_paint){
        .t = state->t.x, .grow = state->hover, .focus = state->focus,
        .fs = cui__fs(ui, ui->theme.text_lg), .pa = (int)lroundf(*value * 100), .names = names,
        .a = cui__frame_strdup(ui, first, first_length), .alen = first_length,
        .b = cui__frame_strdup(ui, second, second_length), .blen = second_length,
        .a_bg = cui_alpha(color, .18f),
        .a_name = cui__legible(cui_mix(color, CUI_C(ui, BACKGROUND), .25f), under, 3),
        .a_pct = cui__legible(color, under, 4.5f),
        .b_bg = track, .b_name = cui__legible(CUI_C(ui, TEXT_MUTED), track, 3),
        .b_pct = CUI_C(ui, FOREGROUND), .bar = CUI_C(ui, FOREGROUND), .rtl_out = &state->rtl
    };
    native->paint = cui__paint_ratio;
    native->paint_data = paint;
    fpas_creative_node(ui, node, key);
    parts[0] = node;
    return enabled && *value != old;
}
