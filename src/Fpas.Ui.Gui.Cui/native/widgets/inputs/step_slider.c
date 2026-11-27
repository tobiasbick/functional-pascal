// [Documentation](../../../../../docs/ui/gui/creative-controls/step-slider.md)
#include "../../fpas_cui.h"

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
    // CUI-100 (docs/future/cui-upstream-bugs.md#cui-100): apply the recorded release position.
    const fpas_slider_release release = fpas_slider_release_begin(ui, track_key, enabled);
    // CUI-107 (docs/future/cui-upstream-bugs.md#cui-107): keep activation keys in the open popup.
    const bool covered_tool = tool && (ui->focus == tool_key || ui->focus_next_request == tool_key) && cui__keys_covered(ui);
    const fpas_well_keys keys = fpas_well_pressed(ui);
    if (covered_tool) fpas_well_set_keys(ui, (fpas_well_keys){0});
    const int result = cui_step_slider_ex(ui, label, value, minimum, maximum, &options);
    if (covered_tool) fpas_well_set_keys(ui, keys);
    fpas_slider_release_end(ui, release);
    parts[0] = fpas_find(ui, track_key);
    parts[1] = tool ? fpas_find(ui, tool_key) : -1;
    fpas_well_literal(ui, node, literal);
    fpas_creative_node(ui, node, key);
    return enabled ? result : 0;
}
