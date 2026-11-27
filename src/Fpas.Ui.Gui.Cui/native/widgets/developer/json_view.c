// [Documentation](../../../../../docs/ui/gui/code/json-viewer.md)
#include "../../fpas_cui.h"

/** Draws the native document and applies the host's input permission to its internal viewport. */
int fpas_cui_json_view(cui_ctx *ui, const char *id, const char *json, int depth, float height, int enabled) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const cui_id scroll = key ^ 0x15011u;
    const bool allowed = enabled && !cui_is_disabled(ui);
    fpas_document_scroll_input(ui, scroll, allowed);
    const cui_json_opts options = { .expand_depth = depth, .max_height = height };
    cui_push_disabled(ui, !enabled);
    const bool valid = cui_json_view_ex(ui, id, json, &options);
    cui_pop_disabled(ui);
    const int viewport = ui->nodes[node].first;
    if (viewport >= 0 && ui->nodes[viewport].id == scroll)
        ui->nodes[viewport].paint = allowed ? fpas_document_scroll_layout : fpas_disabled_document_scroll_layout;
    fpas_own(ui, key, scroll, scroll ^ 0x5C5C5C5Cu, 0);
    ui->last_node = node;
    ui->last_id = key;
    return valid;
}
