#include "../fpas_cui.h"

int fpas_cui_status_tag(cui_ctx *ui, const char *id,
    const char *label, int status, const char *icon, int tone, int flags, int focus, int *parts) {
    const bool enabled = (flags & 4) && !cui_is_disabled(ui);
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    if (enabled && focus == 0) ui->focus_next_request = key;
    const cui_status_tag_opts options = { .id = id, .size = (flags & 1) ? CUI_SIZE_SM : CUI_SIZE_MD,
        .removable = (flags & 2) != 0, .icon = icon, .tone = tone };
    cui_push_disabled(ui, !enabled);
    const bool removed = cui_status_tag_ex(ui, label, status, &options);
    cui_pop_disabled(ui);
    if (label && strstr(label, "##")) fpas_text(ui, fpas_label_node(ui, node), label);
    if ((flags & 2) && !enabled) ui->nodes[node].opacity = CUI__DISABLED;
    parts[0] = node;
    fpas_creative_node(ui, node, key);
    return enabled && removed;
}
