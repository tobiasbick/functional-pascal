#include "../fpas_cui.h"

static void fpas_calendar_part(cui_ctx *ui, cui_id key, int first, int part,
    int *parts, int *nodes, int *count) {
    const int node = fpas_find(ui, key);
    if (node < first || *count >= FPAS_CALENDAR_PARTS) return;
    parts[*count] = part;
    nodes[(*count)++] = node;
}

static void fpas_calendar_parts(cui_ctx *ui, cui_id key, int first,
    int *parts, int *nodes, int *count) {
    fpas_calendar_part(ui, fpas_calendar_navigation(key, false), first, -8, parts, nodes, count);
    fpas_calendar_part(ui, fpas_calendar_navigation(key, true), first, -9, parts, nodes, count);
    fpas_calendar_part(ui, cui__hash_str("#title", key), first, -10, parts, nodes, count);
    const cui__cal_state *state = cui_state(ui, key, sizeof *state);
    for (int k = 0; k < 12; ++k) {
        int cell = state->view_y * 100 + k + 1;
        fpas_calendar_part(ui, cui__hash(&cell, sizeof cell, key ^ 0x9E3779B9u), first,
            -100 - state->view_y * 12 - k, parts, nodes, count);
        cell = -(state->year0 + k);
        fpas_calendar_part(ui, cui__hash(&cell, sizeof cell, key ^ 0x9E3779B9u), first,
            -200000 - state->year0 - k, parts, nodes, count);
    }
    /* Inspect after drawing: title and navigation clicks can change the view. */
    const int month = cui_date_to_days((cui_date){state->view_y, state->view_m, 1});
    for (int day = month - 38; day < month + 74; ++day) {
        const int number = day + FPAS_DATE_EPOCH;
        if (number < 0 || number > FPAS_MAX_DAY_NUMBER) continue;
        const cui_date date = cui_date_from_days(day);
        const int cell = date.year * 10000 + date.month * 100 + date.day;
        fpas_calendar_part(ui, cui__hash(&cell, sizeof cell, key), first, number, parts, nodes, count);
    }
}

int fpas_cui_calendar(cui_ctx *ui, const char *id, int value, int enabled, int focus,
    int *root, int *parts, int *nodes, int *count) {
    const cui_id key = cui_make_id(ui, id);
    cui_date date = fpas_date_from_number(value);
    const int first = ui->node_count;
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (allowed && focus) {
        ui->focus_next_request = focus == 3 ? cui__hash_str("#title", key)
            : fpas_calendar_navigation(key, focus == 2);
    }
    cui_push_disabled(ui, !enabled);
    const bool changed = cui_calendar(ui, id, &date);
    cui_pop_disabled(ui);

    *root = fpas_find(ui, key);
    ui->last_node = *root;
    ui->last_id = key;
    *count = 0;
    if (parts) fpas_calendar_parts(ui, key, first, parts, nodes, count);
    const int proposed = fpas_date_number(date);
    return allowed && changed && proposed >= 0 ? proposed : value;
}
