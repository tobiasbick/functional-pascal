// [Documentation](../../../../../docs/ui/gui/planning/week-calendar.md)
#include "../../fpas_cui.h"

int fpas_cui_week_calendar(cui_ctx *ui, const char *id, int week, fpas_week_event *events, int count,
    float start, float finish, float hour_height, int height_kind, float height, int flags, int focus, int *parts) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    if (enabled && focus >= 0 && focus < count)
        ui->focus_next_request = fpas_well_part(key ^ 0xE7u, focus);
    cui_cal_event *native = cui__frame_alloc(ui, sizeof *native * (size_t)CUI_MAX(count, 1));
    for (int i = 0; i < count; ++i) {
        const fpas_week_event *e = &events[i];
        native[i] = (cui_cal_event){ e->title, e->detail, e->day, e->start, e->finish, e->tone };
    }
    const cui_week_opts options = { .start_hour = start, .end_hour = finish, .hour_height = hour_height,
        .height = fpas_extent(height_kind, height), .read_only = (flags & 2) != 0 || !enabled };
    cui_push_disabled(ui, !enabled);
    int clicked = cui_week_calendar_ex(ui, id, fpas_date_from_number(week), native, count, &options);
    cui_pop_disabled(ui);
    for (int i = 0; i < count; ++i) {
        events[i].day = native[i].day;
        events[i].start = native[i].start;
        events[i].finish = native[i].end;
        parts[i] = fpas_find(ui, fpas_well_part(key ^ 0xE7u, i));
    }
    fpas_own(ui, key, key ^ 0x7EE4u, key ^ 0x6E1Du, 0);
    fpas_creative_node(ui, node, key);
    return enabled ? clicked : -1;
}
