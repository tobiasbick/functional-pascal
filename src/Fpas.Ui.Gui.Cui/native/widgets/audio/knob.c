// [Documentation](../../../../../docs/ui/gui/audio/knob.md)
#include "../../fpas_cui.h"

static const cui_key fpas_dial_adjustments[] = {
    CUI_KEY_UP, CUI_KEY_RIGHT, CUI_KEY_DOWN, CUI_KEY_LEFT, CUI_KEY_PAGE_UP, CUI_KEY_PAGE_DOWN,
    CUI_KEY_HOME, CUI_KEY_END, CUI_KEY_DELETE, CUI_KEY_BACKSPACE
};

/** Reads input before constructing the native dial. */
static cui_interact fpas_dial_interaction(cui_ctx *ui, cui_id key, bool allowed) {
    return allowed ? fpas_probe_drag_interaction(ui, key) : (cui_interact){0};
}

/** Checks for a keyboard adjustment in this frame. */
static bool fpas_dial_keys(cui_ctx *ui) {
    for (size_t i = 0; i < CUI_COUNTOF(fpas_dial_adjustments); ++i)
        if (cui_key_pressed(ui, fpas_dial_adjustments[i])) return true;
    return false;
}

/** Keeps the dial wheel hit valid only while the same control owns focus. */
static bool fpas_dial_wheel(cui_ctx *ui, cui_id wheel, cui_interact input) {
    if (ui->scroll_target != wheel) return false;
    // CUI-118 (docs/future/cui-upstream-bugs.md#cui-118): wheel hits come from the previous paint.
    if (!input.keyboard_focused) {
        ui->scroll_target = 0;
        return false;
    }
    return ui->scroll_dx != 0 || ui->scroll_dy != 0;
}

/** Leaves navigation keys available to an overlay that owns the keyboard. */
static unsigned fpas_dial_mute_keys(cui_ctx *ui, cui_interact input) {
    unsigned saved = 0;
    if (input.focused && !input.keyboard_focused) {
        for (size_t i = 0; i < CUI_COUNTOF(fpas_dial_adjustments); ++i) {
            if (!ui->kpressed[fpas_dial_adjustments[i]]) continue;
            saved |= 1u << i;
            ui->kpressed[fpas_dial_adjustments[i]] = 0;
        }
    }
    return saved;
}

/** Restores keys hidden from the dial during native construction. */
static void fpas_dial_restore_keys(cui_ctx *ui, unsigned saved) {
    for (size_t i = 0; i < CUI_COUNTOF(fpas_dial_adjustments); ++i)
        if (saved & (1u << i)) ui->kpressed[fpas_dial_adjustments[i]] = 1;
}

/** Draws the native dial and returns only value proposals caused by input. */
int fpas_cui_knob(cui_ctx *ui, const char *id, const char *label, float *value,
    float minimum, float maximum, float step, float reset, const char *format, const char *unit,
    int hide_value, float size, int bipolar, int log_scale, int palette, uint32_t rgba, int enabled, int focus,
    int *control) {
    const cui_id group = cui_make_id(ui, id), key = group ^ 0x4B0Bu, wheel = group ^ 0x4B0Cu;
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (allowed && focus) ui->focus_next_request = key;
    const cui__knob_state *before = cui__state_lazy(ui, group, sizeof *before, false);
    const bool continued = allowed && before && before->dragging && ui->active == key;
    const bool released = allowed && fpas_drag_release(ui, key, continued, NULL);
    const cui_id hot = ui->hot;
    if (released && !continued) ui->hot = key;
    const cui_interact input = fpas_dial_interaction(ui, key, allowed);
    const bool wheel_input = fpas_dial_wheel(ui, wheel, input);
    const bool key_input = input.keyboard_focused && fpas_dial_keys(ui);
    const bool reset_key = key_input && (cui_key_pressed(ui, CUI_KEY_DELETE) || cui_key_pressed(ui, CUI_KEY_BACKSPACE));
    const bool end_key = key_input && !reset_key && cui_key_pressed(ui, CUI_KEY_END);
    const bool home_key = key_input && !reset_key && cui_key_pressed(ui, CUI_KEY_HOME);
    const bool fine = (ui->mods & CUI_MOD_SHIFT) != 0;
    const bool keep_anchor = continued && fine == before->fine;
    const float anchor = keep_anchor ? before->ref_y : continued ? ui->pmy : ui->press_y;
    const bool held = continued && input.active && ui->mdown[CUI_MOUSE_LEFT] && !ui->mpressed[CUI_MOUSE_LEFT];
    const bool moved = held && (ui->my != anchor || ui->my != ui->pmy);
    const bool release_move = released && ui->release_y != (continued ? ui->pmy : ui->press_y);
    const bool adjusted = wheel_input || key_input || moved || release_move || input.double_clicked;
    const char *value_format = format ? format : "%.0f";
    const bool literal_label = strstr(label, "##") != NULL;
    const cui_knob_opts options = {
        .label = cui__frame_strdup(ui, literal_label && !cui__display_len(label) ? "x" : label, -1),
        // CUI-116 (docs/future/cui-upstream-bugs.md#cui-116): End reaches a non-grid endpoint.
        .format = value_format, .unit = unit, .step = adjusted && !home_key && !end_key ? step : 0,
        .reset = reset,
        .disabled = !enabled, .hide_value = hide_value != 0,
        .size = size, .bipolar = bipolar != 0, .log_scale = log_scale != 0,
        .color = fpas_chart_color(palette, rgba)
    };
    float next = *value;
    // CUI-100 (docs/future/cui-upstream-bugs.md#cui-100): use the recorded vertical release position.
    if (release_move) {
        const float position = keep_anchor ? before->ref_t : cui__dl_pos(*value, minimum, maximum, log_scale != 0);
        next = cui__dl_value(position + (anchor - ui->release_y) / (fine ? 2000.0f : 200.0f),
            minimum, maximum, options.step, log_scale != 0);
    }
    const unsigned keys = fpas_dial_mute_keys(ui, input);
    cui_knob_ex(ui, id, &next, minimum, maximum, &options);
    fpas_dial_restore_keys(ui, keys);
    ui->hot = hot;
    const int root = fpas_find(ui, group);
    *control = fpas_find(ui, key);
    if (literal_label) {
        const int caption = ui->nodes[root].last;
        fpas_text(ui, caption, label);
        const float width = ceilf(cui__text_width(ui, CUI_FONT_BODY, ui->nodes[caption].font_size, label, -1, false));
        const float diameter = ui->nodes[*control].w.value;
        ui->nodes[root].w = CUI_PX(CUI_MAX(ui->nodes[root].w.value, CUI_MIN(width, diameter * 2)));
    }
    if (!hide_value && ((unit && strstr(unit, "##")) || strstr(value_format, "##")))
        fpas_text(ui, ui->nodes[*control].next, cui__dl_text(ui, value_format, unit, next));
    ui->last_node = root;
    fpas_own(ui, key, group, wheel, 0);
    if (!wheel_input && (end_key || home_key)) next = end_key ? maximum : minimum;
    if (!adjusted || !isfinite(next) || next == *value) return 0;
    *value = next;
    return 1;
}
