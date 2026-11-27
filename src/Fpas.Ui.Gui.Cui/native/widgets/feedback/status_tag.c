// [Documentation](../../../../../docs/ui/gui/creative-controls/status-tag.md)
#include "../../fpas_cui.h"

/** Presents a status tag and reports removal only when it accepts input. */
int fpas_cui_status_tag(cui_ctx *ui, const char *id,
    const char *label, int status, const char *icon, int tone, int flags, int focus, int *parts) {
    const bool removable = (flags & 2) != 0;
    const bool enabled = removable && (flags & 4) && !cui_is_disabled(ui);
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    if (enabled && focus == 0) ui->focus_next_request = key;
    const cui_status_tag_opts options = { .id = id, .size = (flags & 1) ? CUI_SIZE_SM : CUI_SIZE_MD,
        .removable = removable, .icon = icon, .tone = tone };
    const fpas_well_keys keys = fpas_well_pressed(ui);
    /* CUI-107 (docs/future/cui-upstream-bugs.md#cui-107): covered tags still report keyboard activation. */
    const bool covered = enabled && (keys.enter || keys.space) && cui__keys_covered(ui);
    cui_push_disabled(ui, !enabled);
    if (covered) fpas_well_set_keys(ui, (fpas_well_keys){ .left = keys.left, .right = keys.right });
    const bool removed = cui_status_tag_ex(ui, label, status, &options);
    if (covered) fpas_well_set_keys(ui, keys);
    cui_pop_disabled(ui);
    if (label && strstr(label, "##")) fpas_text(ui, fpas_label_node(ui, node), label);
    if (removable && !enabled) ui->nodes[node].opacity = CUI__DISABLED;
    parts[0] = node;
    fpas_creative_node(ui, node, key);
    return enabled && removed;
}
