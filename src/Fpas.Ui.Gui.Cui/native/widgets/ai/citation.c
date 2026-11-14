#include "../../fpas_cui.h"

int fpas_cui_citation(cui_ctx *ui, const char *id, int number, const cui_source *source, int flags) {
    const int root = ui->node_count;
    cui_push_id(ui, id);
    char name[24];
    snprintf(name, sizeof name, "cite%d", number);
    const cui_id key = cui_make_id(ui, name);
    const bool enabled = fpas_ai_enabled(ui, flags, key);
    cui_push_disabled(ui, !enabled);
    const bool picked = cui_citation(ui, number, source);
    cui_pop_disabled(ui);
    cui_pop_id(ui);
    fpas_creative_node(ui, root, key);
    return enabled && picked;
}
