// [Documentation](../../../../../docs/ui/gui/ai/agent-status.md)
#include "../../fpas_cui.h"

/** Draws the passive status pill, including a still indicator under reduced motion. */
void fpas_cui_agent_status(cui_ctx *ui, const char *id, const char *label, int status, float seconds, const char *icon) {
    const int root = ui->node_count;

    const cui_id key = cui_make_id(ui, id);
    const cui_agent_status_opts opts = { .status = (cui_task_status)status, .seconds = seconds, .icon = icon };
    const bool animating = ui->animating;
    cui_agent_status_ex(ui, label, &opts);
    if (status == CUI_TASK_RUNNING && cui__still(ui))
        fpas_freeze_loader(ui, ui->nodes[root].first, animating);

    fpas_creative_node(ui, root, key);
}
