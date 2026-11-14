#include "../../fpas_cui.h"

static void fpas_countdown_retire(cui_ctx *ui, cui_id key) {
    for (int unit = 0; unit < 4; ++unit)
        fpas_forget_state(ui, cui__hash(&unit, sizeof unit, key));
}

int fpas_cui_countdown(cui_ctx *ui, const char *id, double seconds_left, int compact, int hide_days) {
    const cui_id key = cui_make_id(ui, id);
    fpas_on_forget(ui, key, fpas_countdown_retire);
    const cui_countdown_opts options = { .compact = compact != 0, .hide_days = hide_days != 0 };
    const int node = ui->node_count;
    cui_countdown_ex(ui, id, seconds_left, &options);
    ui->last_node = node;
    return node;
}
