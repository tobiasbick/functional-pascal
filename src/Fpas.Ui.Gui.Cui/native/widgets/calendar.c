#include "../fpas_cui.h"

static int fpas_calendar_month(cui_ctx *ui, cui_id key, cui_date value) {
    cui__cal_state *state = cui_state(ui, key, sizeof *state);
    cui_date month = state->init ? (cui_date){state->view_y, state->view_m, 1}
        : value.year ? value : cui_date_today();
    month.day = 1;
    return cui_date_to_days(month);
}

static void fpas_calendar_part(cui_ctx *ui, cui_id key, int first, int part,
    int *parts, int *nodes, int *count) {
    const int node = fpas_find(ui, key);
    if (node < first || *count >= FPAS_CALENDAR_PARTS) return;
    parts[*count] = part;
    nodes[(*count)++] = node;
}

static void fpas_calendar_parts(cui_ctx *ui, const char *id, cui_id key, int first, int month,
    int *parts, int *nodes, int *count) {
    cui_push_id(ui, id);
    fpas_calendar_part(ui, fpas_calendar_month_button(ui, false), first, -8, parts, nodes, count);
    fpas_calendar_part(ui, fpas_calendar_month_button(ui, true), first, -9, parts, nodes, count);
    cui_pop_id(ui);
    /* A month click can move the grid one month in either direction during drawing. */
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
    const int month = fpas_calendar_month(ui, key, date);
    const int first = ui->node_count;
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (allowed && focus) {
        cui_push_id(ui, id);
        ui->focus_next_request = fpas_calendar_month_button(ui, focus == 2);
        cui_pop_id(ui);
    }
    cui_push_disabled(ui, !enabled);
    const bool changed = cui_calendar(ui, id, &date);
    cui_pop_disabled(ui);

    *root = fpas_find(ui, key);
    ui->last_node = *root;
    ui->last_id = key;
    *count = 0;
    if (parts) fpas_calendar_parts(ui, id, key, first, month, parts, nodes, count);
    const int proposed = fpas_date_number(date);
    return allowed && changed && proposed >= 0 ? proposed : value;
}
