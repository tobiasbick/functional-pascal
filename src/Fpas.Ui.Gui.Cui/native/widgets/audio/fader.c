// [Documentation](../../../../../docs/ui/gui/audio/fader.md)
#include "../../fpas_cui.h"

/** Applies the last drag position using CUI's existing relative drag anchor. */
static float fpas_fader_release(cui_ctx *ui, const cui__fader_state *state, float position,
    bool horizontal, float start, float end) {
    const bool fine = (ui->mods & CUI_MOD_SHIFT) != 0;
    const float release = horizontal ? ui->release_x : ui->release_y;
    float reference, value;
    if (state && state->dragging) {
        reference = state->ref_px;
        value = state->ref_t;
        if (fine != state->fine) {
            reference = horizontal ? ui->pmx : ui->pmy;
            value = position;
        }
    } else {
        reference = horizontal ? ui->press_x : ui->press_y;
        const float thumb = start + (end - start) * position;
        value = fabsf(reference - thumb) > CUI__FADER_TH * 0.5f + 2
            ? cui__clampf((reference - start) / (end - start), 0, 1) : position;
    }
    return cui__clampf(value + (release - reference) / (end - start) * (fine ? .1f : 1), 0, 1);
}

int fpas_cui_fader(cui_ctx *ui, const char *id, const char *label, float *value,
    float minimum, float maximum, float step, float reset, const char *format, const char *unit,
    int hide_value, float length, int horizontal, const float *ticks, int tick_count,
    const float *levels, int channels, int enabled, int focus, int *control) {
    const cui_id group = cui_make_id(ui, id), key = group ^ 0xFAD0u, wheel = group ^ 0xFAD1u;
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (allowed && focus) ui->focus_next_request = key;
    const cui_interact input = fpas_dial_interaction(ui, key, allowed);
    const bool keys = input.keyboard_focused && fpas_dial_keys(ui);
    const bool wheel_input = fpas_dial_wheel(ui, wheel, input);
    const cui_rect bounds = cui__prev_rect(ui, key);
    float start, end, cross;
    cui__fader_travel(bounds, horizontal, 0, &start, &end, &cross);
    const bool has_track = bounds.w > 0 && fabsf(end - start) > 1;
    const cui__fader_state *state = cui__state_lazy(ui, group, sizeof *state, false);
    const bool dragging = state && state->dragging && ui->active == key;
    // CUI-100 (docs/future/cui-upstream-bugs.md#cui-100): apply a queued drag release.
    const bool release = allowed && has_track && fpas_drag_release(ui, key, dragging, NULL);
    const bool drag = allowed && has_track && input.active && ui->mdown[CUI_MOUSE_LEFT];
    const bool adjusting = release || drag || keys || wheel_input || input.double_clicked;
    const bool reset_key = keys && (cui_key_pressed(ui, CUI_KEY_DELETE) || cui_key_pressed(ui, CUI_KEY_BACKSPACE));
    // CUI-116 (docs/future/cui-upstream-bugs.md#cui-116): Home and End select exact bounds.
    const bool endpoint = keys && !reset_key && (cui_key_pressed(ui, CUI_KEY_HOME) || cui_key_pressed(ui, CUI_KEY_END));
    const float position = cui__dl_norm(*value, minimum, maximum);
    const float released = release
        ? fpas_fader_release(ui, dragging ? state : NULL, position, horizontal, start, end) : position;
    if (release && !dragging && !input.focused) {
        ui->focus_next_request = key;
        ui->focus_visible = false;
    }
    const bool literal_label = strstr(label, "##") != NULL;
    const char *number_format = format ? format : "%.1f";
    const cui_fader_opts options = {
        .label = cui__frame_strdup(ui, literal_label && !cui__display_len(label) ? "x" : label, -1),
        .format = number_format, .unit = unit, .step = adjusting && !endpoint ? step : 0, .reset = reset,
        .disabled = !enabled, .hide_value = hide_value != 0,
        .length = length, .horizontal = horizontal != 0, .ticks = ticks, .tick_count = tick_count,
        .meter = levels, .meter_channels = channels
    };
    float next = release ? cui__dl_value(released, minimum, maximum, 0, false) : *value;
    const uint32_t muted_keys = fpas_dial_mute_keys(ui, input);
    cui_fader_ex(ui, id, &next, minimum, maximum, &options);
    fpas_dial_restore_keys(ui, muted_keys);
    if (endpoint && !wheel_input) next = cui_key_pressed(ui, CUI_KEY_END) ? maximum : minimum;
    const int root = fpas_find(ui, group);
    *control = fpas_find(ui, key);
    if (literal_label) fpas_text(ui, (horizontal ? ui->nodes[ui->nodes[root].first].first : ui->nodes[root].last), label);
    if (!horizontal && !hide_value && (strstr(number_format, "##") || (unit && strstr(unit, "##"))))
        fpas_text(ui, ui->nodes[root].first, cui__dl_text(ui, number_format, unit, next));
    ui->last_node = root;
    fpas_own(ui, key, group, wheel, 0);
    if (!allowed || !adjusting || !isfinite(next) || next == *value) return 0;
    *value = next;
    return 1;
}
