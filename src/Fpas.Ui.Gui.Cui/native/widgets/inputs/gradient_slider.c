#include "../../fpas_cui.h"

int fpas_cui_gradient_slider(cui_ctx *ui, const char *id,
    const char *label, float *value, float minimum, float maximum, int track, const float *positions, const uint32_t *colors, int count, int palette, uint32_t rgba, float step, const char *format, const char *display, const char *tool, const char *tooltip, int flags, int focus, uint32_t *sample, int *parts) {
    const int node = ui->node_count;
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    const char *literal = label;
    label = fpas_well_label(ui, label, id);
    const cui_id key = cui_make_id(ui, label);
    const cui_id track_key = key ^ 0x67A5u, tool_key = key ^ CUI__WELL_TOOL_SALT;
    if (enabled && focus == 0) ui->focus_next_request = track_key;
    if (enabled && focus == 1 && tool) ui->focus_next_request = tool_key;
    const float previous = *value;
    const bool pressed = ui->mpressed[CUI_MOUSE_LEFT];
    const cui_rect previous_track = cui__prev_rect(ui, track_key);
    const bool pointer = ui->mdown[CUI_MOUSE_LEFT] && previous_track.w > 2 * CUI__GS_PAD &&
        (ui->active == track_key || (pressed && ui->hot == track_key && ui->active == 0));
    const bool keyboard = (!pressed || ui->hot == track_key) && fpas_well_keys(ui, track_key);
    const bool endpoint = keyboard &&
        (cui_key_pressed(ui, CUI_KEY_HOME) || cui_key_pressed(ui, CUI_KEY_END));
    const bool arrow = keyboard &&
        (cui_key_pressed(ui, CUI_KEY_LEFT) || cui_key_pressed(ui, CUI_KEY_RIGHT) ||
         cui_key_pressed(ui, CUI_KEY_UP) || cui_key_pressed(ui, CUI_KEY_DOWN));
    /* CUI-031 (docs/future/cui-upstream-bugs.md#cui-031): snapping a
       passive value or a keyboard endpoint can pull it inside the range. */
    if (!enabled || endpoint || (!pointer && !arrow)) step = 0;
    cui_gradient_stop *stops = count ? cui__frame_alloc(ui, (size_t)count * sizeof *stops) : NULL;
    for (int i = 0; i < count; ++i)
        stops[i] = (cui_gradient_stop){ positions[i], fpas_chart_color(-1, colors[i]) };
    cui_color under;
    const cui_gradient_slider_opts options = { .track = track, .stops = stops, .stop_count = count,
        .color = fpas_chart_color(palette, rgba), .color_out = &under, .step = step,
        .format = format, .value = display, .swatch = (flags & 2) != 0, .tool = tool, .tool_tooltip = tooltip,
        .tool_on = (flags & 4) != 0, .plain = (flags & 8) != 0, .disabled = !enabled };
    const int result = cui_gradient_slider_ex(ui, label, value, minimum, maximum, &options);
    if (!enabled || (!pointer && !arrow && !endpoint)) *value = previous;
    *sample = fpas_well_rgba(under);
    parts[0] = fpas_find(ui, track_key);
    parts[1] = tool ? fpas_find(ui, tool_key) : -1;
    fpas_well_literal(ui, node, literal);
    fpas_creative_node(ui, node, key);
    return enabled ? ((result & CUI_WELL_ACTION) | (*value != previous ? CUI_WELL_CHANGED : 0)) : 0;
}
