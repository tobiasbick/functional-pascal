// [Documentation](../../../../../docs/ui/gui/code/json-viewer.md)
#include "../../fpas_cui.h"

int fpas_cui_json_view(cui_ctx *ui, const char *id, const char *json, int depth, float height, int enabled) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const cui_json_opts options = { .expand_depth = depth, .max_height = height };
    cui_push_disabled(ui, !enabled);
    const bool valid = cui_json_view_ex(ui, id, json, &options);
    cui_pop_disabled(ui);
    fpas_own(ui, key, key ^ 0x15011u, 0, 0);
    ui->last_node = node;
    ui->last_id = key;
    return valid;
}
