// [Documentation](../../../../../docs/ui/gui/creative-controls/option-action.md)
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
    const cui_id tool = icon ? key ^ CUI__WELL_TOOL_SALT : 0;
    const cui_option_action_opts options = { .tooltip = tooltip, .action_on = (flags & 4) != 0,
        .action_disabled = !(flags & 16), .value = display, .plain = (flags & 8) != 0 };
    const char **captions = cui__frame_alloc(ui, (size_t)CUI_MAX(count, 1) * sizeof *captions);
    for (int i = 0; i < count; ++i)
        captions[i] = strstr(items[i], "##") ? "x" : items[i];
    fpas_own(ui, key, row, tool, 0);
    fpas_indexed_focus(ui, row, count, 0);
    fpas_well_focus(ui, enabled, row, key, count, tool && !options.action_disabled, focus);
    const fpas_well_keys keys = fpas_well_pressed(ui);
    /* CUI-156 (docs/future/cui-upstream-bugs.md#cui-156): follow the displayed order in RTL. */
    const bool rtl = (keys.left || keys.right) && count > 1 &&
        cui__prev_rect(ui, fpas_well_part(row, 0)).x > cui__prev_rect(ui, fpas_well_part(row, 1)).x;
    /* Leave choice and action keys to the popup.
       CUI-107 (docs/future/cui-upstream-bugs.md#cui-107), CUI-131 (docs/future/cui-upstream-bugs.md#cui-131). */
    const bool covered = cui__keys_covered(ui);
    if (covered) fpas_well_set_keys(ui, (fpas_well_keys){0});
    else if (rtl) fpas_well_set_keys(ui, (fpas_well_keys){keys.right, keys.left, keys.enter, keys.space});
    cui_push_disabled(ui, !enabled);
    const int result = cui_option_action_ex(ui, label, captions, count, value, icon, &options);
    cui_pop_disabled(ui);
    if (rtl || covered) fpas_well_set_keys(ui, keys);
    for (int i = 0; i < count; ++i) {
        parts[i] = fpas_find(ui, fpas_well_part(row, i));
        if (parts[i] >= 0) {
            if (captions[i] != items[i]) fpas_text(ui, fpas_label_node(ui, parts[i]), items[i]);
            if (enabled) fpas_well_choice_focus(ui, parts[i]);
        }
    }
    parts[count] = tool ? fpas_find(ui, tool) : -1;
    fpas_well_literal(ui, node, literal);
    fpas_creative_node(ui, node, key);
    return enabled ? result : 0;
}
