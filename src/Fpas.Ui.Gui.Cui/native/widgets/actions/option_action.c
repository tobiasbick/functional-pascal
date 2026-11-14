#include "../../fpas_cui.h"

int fpas_cui_option_action(cui_ctx *ui, const char *id,
    const char *label, const char *const *items, int count, int *value, const char *icon, const char *tooltip, const char *display, int flags, int focus, int *parts) {
    const int node = ui->node_count;
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    const char *literal = label;
    label = fpas_well_label(ui, label, id);
    const cui_id key = cui_make_id(ui, label);
    cui_push_id(ui, label);
    const cui_id row = cui_make_id(ui, "choice");
    cui_pop_id(ui);
    const cui_option_action_opts options = { .tooltip = tooltip, .action_on = (flags & 4) != 0,
        .action_disabled = !(flags & 16), .value = display, .plain = (flags & 8) != 0 };
    fpas_well_focus(ui, enabled, row, key, count, !options.action_disabled, focus);
    cui_push_disabled(ui, !enabled);
    const int result = cui_option_action_ex(ui, label, items, count, value, icon, &options);
    cui_pop_disabled(ui);
    for (int i = 0; i < count; ++i) {
        parts[i] = fpas_find(ui, fpas_well_part(row, i));
        if (parts[i] >= 0 && strstr(items[i], "##")) fpas_text(ui, fpas_label_node(ui, parts[i]), items[i]);
    }
    parts[count] = icon ? fpas_find(ui, key ^ CUI__WELL_TOOL_SALT) : -1;
    fpas_own(ui, key, row, 0, 0);
    fpas_well_literal(ui, node, literal);
    fpas_creative_node(ui, node, key);
    return enabled ? result : 0;
}
