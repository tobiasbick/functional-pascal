#include "../fpas_cui.h"

void fpas_cui_agent_status(cui_ctx *ui, const char *id, const char *label, int status, float seconds, const char *icon) {
    const int root = ui->node_count;

    const cui_id key = cui_make_id(ui, id);
    const cui_agent_status_opts opts = { .status = (cui_task_status)status, .seconds = seconds, .icon = icon };
    cui_agent_status_ex(ui, label, &opts);

    fpas_creative_node(ui, root, key);
}
