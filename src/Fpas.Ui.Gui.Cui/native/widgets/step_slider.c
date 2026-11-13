#include "../fpas_cui.h"

int fpas_cui_step_slider(cui_ctx *ui, const char *id,
    const char *label, int *value, int minimum, int maximum, const char *const *labels, int count, const char *format, const char *display, const char *tool, const char *tooltip, int flags, int focus, int *parts) {
    const int node = ui->node_count;
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    const char *literal = label;
    label = fpas_well_label(ui, label, id);
    const cui_id key = cui_make_id(ui, label);
    const cui_id track_key = key ^ 0x57E9u, tool_key = key ^ CUI__WELL_TOOL_SALT;
    if (enabled && focus == 0) ui->focus_next_request = track_key;
    if (enabled && focus == 1 && tool) ui->focus_next_request = tool_key;
    const cui_step_slider_opts options = { .labels = count ? labels : NULL, .format = format, .value = display,
        .tool = tool, .tool_tooltip = tooltip, .tool_on = (flags & 4) != 0,
        .plain = (flags & 8) != 0, .disabled = !enabled };
    const int result = cui_step_slider_ex(ui, label, value, minimum, maximum, &options);
    parts[0] = fpas_find(ui, track_key);
    parts[1] = tool ? fpas_find(ui, tool_key) : -1;
    fpas_well_literal(ui, node, literal);
    fpas_creative_node(ui, node, key);
    return enabled ? result : 0;
}
