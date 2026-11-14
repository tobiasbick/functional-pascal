#include "../../fpas_cui.h"

static bool fpas_range_propose(cui_date_range range, int *start, int *end) {
    const int a = fpas_date_number(range.start), b = fpas_date_number(range.end);
    if ((range.start.month && a < 0) || (range.end.month && b < 0)
        || a > FPAS_MAX_DAY_NUMBER || b > FPAS_MAX_DAY_NUMBER) return false;
    if (*start == a && *end == b) return false;
    *start = a;
    *end = b;
    return true;
}

static int fpas_range_prepare(cui_ctx *ui, cui_id key, cui_date_range range) {
    cui__rcal_state *state = cui_state(ui, key, sizeof *state);
    /* The same model can be edited by an inline calendar and a popup. */
    state->picking = range.start.year != 0 && range.end.year == 0;
    cui_date month = state->init ? (cui_date){state->view_y, state->view_m, 1}
        : range.start.year ? range.start : cui_date_today();
    month.day = 1;
    return cui_date_to_days(month);
}

static void fpas_range_part(cui_ctx *ui, cui_id id, int first, int part,
    int *parts, int *nodes, int *count) {
    const int node = fpas_find(ui, id);
    if (node < first || *count >= 72) return;
    parts[*count] = part;
    nodes[*count] = node;
    ++*count;
}

static void fpas_range_parts(cui_ctx *ui, const char *id, cui_id key, int first, int month,
    int *parts, int *nodes, int *count) {
    cui_push_id(ui, id);
    const cui_id previous = fpas_calendar_month_button(ui, false);
    const cui_id next = fpas_calendar_month_button(ui, true);
    fpas_range_part(ui, previous, first, -8, parts, nodes, count);
    fpas_range_part(ui, next, first, -9, parts, nodes, count);
    cui_pop_id(ui);
    /* A navigation click can change the second month while this frame is built. */
    for (int day = month - 31; day < month + 93; ++day) {
        const int number = day + FPAS_DATE_EPOCH;
        if (number < 0 || number > FPAS_MAX_DAY_NUMBER) continue;
        fpas_range_part(ui, cui__hash(&day, sizeof day, key), first, number, parts, nodes, count);
    }
}

int fpas_cui_range_calendar(cui_ctx *ui, const char *id, int *start, int *end,
    int months, int enabled, int *root, int *parts, int *nodes, int *count) {
    const cui_id key = cui_make_id(ui, id);
    cui_date_range range = { fpas_date_from_number(*start), fpas_date_from_number(*end) };
    const int month = fpas_range_prepare(ui, key, range);
    const int first = ui->node_count;
    const bool allowed = enabled && !cui_is_disabled(ui);
    cui_push_disabled(ui, !allowed);
    const bool changed = cui_range_calendar(ui, id, &range, months);
    cui_pop_disabled(ui);
    *root = fpas_find(ui, key);
    ui->last_node = *root;
    ui->last_id = key;
    *count = 0;
    fpas_range_parts(ui, id, key, first, month, parts, nodes, count);
    return allowed && changed && fpas_range_propose(range, start, end);
}
