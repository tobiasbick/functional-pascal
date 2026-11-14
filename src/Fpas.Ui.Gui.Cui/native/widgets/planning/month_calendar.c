#include "../../fpas_cui.h"

int fpas_cui_month_calendar(cui_ctx *ui, const char *id, int *month, int *selected,
    const fpas_month_event *events, int count, int first_weekday, float cell_height,
    const char *const *months, const char *const *days, int today, int flags, int focus, int *parts, int *activated) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const cui_id grid = key ^ 0x6A1Du;
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    if (enabled && focus == 1) ui->focus_next_request = grid;
    cui_push_id(ui, id);
    const cui_id previous = cui__hash_str(CUI_ICON_CHEVRON_LEFT, cui_make_id(ui, "Previous month"));
    const cui_id next = cui__hash_str(CUI_ICON_CHEVRON_RIGHT, cui_make_id(ui, "Next month"));
    const cui_id current = cui_make_id(ui, cui__today_text(ui));
    cui_pop_id(ui);
    if (enabled && focus > 1 && focus <= 4)
        ui->focus_next_request = focus == 2 ? previous : focus == 3 ? next : current;
    cui_month_event *native = cui__frame_alloc(ui, sizeof *native * (size_t)CUI_MAX(count, 1));
    for (int i = 0; i < count; ++i) {
        const fpas_month_event *e = &events[i];
        native[i] = (cui_month_event){ e->title, fpas_date_from_number(e->start),
            fpas_date_from_number(e->finish), e->time, e->all_day != 0, e->tone };
    }
    cui_date shown = fpas_date_from_number(*month);
    cui_date day = fpas_date_from_number(*selected);
    const cui_month_opts options = { .selected = &day, .first_weekday = first_weekday,
        .cell_height = cell_height, .month_names = months, .day_names = days,
        .hour12 = (flags & 2) != 0, .today = fpas_date_from_number(today) };
    cui_push_disabled(ui, !enabled);
    const cui_month_result result = cui_month_calendar_ex(ui, id, &shown, native, count, &options);
    cui_pop_disabled(ui);
    shown.day = 1;
    if (enabled) {
        const int number = fpas_date_number(shown);
        if (number >= 0) *month = number;
        *selected = fpas_date_number(day);
    }
    *activated = enabled ? result.event : -1;
    parts[0] = fpas_find(ui, grid);
    parts[1] = fpas_find(ui, previous);
    parts[2] = fpas_find(ui, next);
    parts[3] = fpas_find(ui, current);
    fpas_own(ui, key, 0, 0, key ^ 0x6A1Eu);
    fpas_creative_node(ui, node, key);
    return enabled ? (result.day_clicked ? 1 : 0) | (result.day_activated ? 2 : 0) | (result.month_changed ? 4 : 0) : 0;
}
