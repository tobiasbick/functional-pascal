#include "../fpas_cui.h"

int fpas_cui_time_dial(cui_ctx *ui, const char *id, int value, float size,
    int enabled, int focus, int *nodes) {
    const cui_id key = cui_make_id(ui, id);
    const bool allowed = enabled && !cui_is_disabled(ui);
    const int hours = 0, minutes = 1;
    const cui_id hour_key = cui__hash(&hours, sizeof hours, key);
    const cui_id minute_key = cui__hash(&minutes, sizeof minutes, key);
    if (allowed && focus) ui->focus_next_request = focus == 1 ? hour_key : minute_key;
    if (!allowed) {
        cui__dial2_state *state = cui__state_lazy(ui, key, sizeof *state, false);
        if (state) state->dragging = false;
    }

    int hour = value / 60, minute = value % 60;
    cui_push_disabled(ui, !enabled);
    cui_time_dial(ui, id, &hour, &minute, size);
    cui_pop_disabled(ui);
    ui->last_node = fpas_find(ui, key);
    if (nodes) {
        nodes[0] = ui->last_node;
        nodes[1] = fpas_find(ui, hour_key);
        nodes[2] = fpas_find(ui, minute_key);
        nodes[3] = fpas_find(ui, key ^ 0xFACEu);
    }
    return allowed ? hour * 60 + minute : value;
}
