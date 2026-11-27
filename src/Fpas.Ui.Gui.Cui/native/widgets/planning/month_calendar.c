// [Documentation](../../../../../docs/ui/gui/planning/month-calendar.md)
#include "../../fpas_cui.h"

/* CUI-080 (docs/future/cui-upstream-bugs.md#cui-080):
   CUI treats negative calendar day indices as unset. Five Gregorian cycles
   keep the CLR date domain positive without changing weekdays or leap days. */
enum { FPAS_MONTH_YEARS = 2000, FPAS_MONTH_EPOCH = 5 * 146097 - FPAS_DATE_EPOCH };

/** Maps a controlled day into the calendar's positive day domain. */
static cui_date fpas_month_date(int day) {
    return day < 0 ? (cui_date){0} : cui_date_from_days(day + FPAS_MONTH_EPOCH);
}

/** Converts a calendar date and rejects proposals outside the CLR range. */
static int fpas_month_number(cui_date date) {
    if (date.year <= FPAS_MONTH_YEARS || date.year > FPAS_MONTH_YEARS + 9999) return -1;
    return cui_date_to_days(date) - FPAS_MONTH_EPOCH;
}

/** Keeps the keyboard cursor inside the month selected by the application.
 * CUI-081 (docs/future/cui-upstream-bugs.md#cui-081). */
static cui__mc_state *fpas_month_focus(cui_ctx *ui, cui_id key, cui_date month,
    cui_date selected, cui_date today, int *previous_selection) {
    cui__mc_state *state = cui_state(ui, key, sizeof *state);
    const int first = cui_date_to_days((cui_date){month.year, month.month, 1});
    const int end = first + cui__mc_days_in(month.year, month.month);
    const int current = selected.year ? cui_date_to_days(selected) : -1;
    const int now = cui_date_to_days(today);
    const bool selected_here = current >= first && current < end;
    if (!state->init) {
        state->hover_ev = state->hover_day = state->more_day = -1;
        state->init = true;
    }
    if (selected_here && current != *previous_selection) state->focus_day = current;
    if (state->focus_day < first || state->focus_day >= end) {
        state->focus_day = selected_here ? current : now >= first && now < end ? now : first;
    }
    *previous_selection = current;
    return state;
}

static const cui_key fpas_month_navigation[] = {
    CUI_KEY_LEFT, CUI_KEY_RIGHT, CUI_KEY_UP, CUI_KEY_DOWN,
    CUI_KEY_PAGE_UP, CUI_KEY_PAGE_DOWN, CUI_KEY_ENTER, CUI_KEY_SPACE
};

typedef struct {
    cui_id hot;
    bool masked[8];
    uint8_t key[8];
} fpas_month_input;

/** Masks only navigation that would leave the DateOnly range. */
static fpas_month_input fpas_month_limit_input(cui_ctx *ui, cui_date shown, const cui__mc_state *state,
    cui_id grid, cui_id previous, cui_id next, int first_weekday) {
    fpas_month_input input = { .hot = ui->hot };
    const bool minimum = shown.year == FPAS_MONTH_YEARS + 1 && shown.month == 1;
    const bool maximum = shown.year == FPAS_MONTH_YEARS + 9999 && shown.month == 12;
    if (!minimum && !maximum) return input;

    const cui_id blocked = minimum ? previous : next;
    const cui_id request = ui->focus_next_request;
    const cui_id focus = request == blocked || request == grid ? request : ui->focus;
    const bool keyboard = focus == grid;
    const int start = FPAS_MONTH_EPOCH, end = FPAS_MONTH_EPOCH + FPAS_MAX_DAY_NUMBER;
    const bool mask[] = {
        keyboard && state->focus_day == start, keyboard && state->focus_day == end,
        keyboard && state->focus_day < start + 7, keyboard && state->focus_day > end - 7,
        keyboard && minimum, keyboard && maximum, focus == blocked, focus == blocked
    };
    for (size_t i = 0; i < CUI_COUNTOF(fpas_month_navigation); ++i) {
        if (!mask[i]) continue;
        input.masked[i] = true;
        input.key[i] = ui->kpressed[fpas_month_navigation[i]];
        ui->kpressed[fpas_month_navigation[i]] = 0;
    }
    if (ui->hot == blocked) ui->hot = 0;
    if (ui->hot == grid) {
        const cui_rect rect = cui__prev_rect(ui, grid);
        if (rect.w > 0 && rect.h > 0) {
            const int first = cui_date_to_days((cui_date){shown.year, shown.month, 1});
            const int lead = (cui__mc_weekday(first) - CUI_CLAMP(first_weekday, 0, 6) + 7) % 7;
            const int rows = (lead + cui__mc_days_in(shown.year, shown.month) + 6) / 7;
            const cui__mc_geo geo = cui__mc_geom(rect, rows);
            const int column = (int)floorf((ui->mx - rect.x) / geo.cw);
            const int row = (int)floorf((ui->my - rect.y) / geo.ch);
            const int day = first - lead + row * 7 + column;
            if (day < start || day > end) ui->hot = 0;
        }
    }
    return input;
}

/** Restores the input made available to subsequent widgets in this frame. */
static void fpas_month_restore_input(cui_ctx *ui, fpas_month_input input) {
    ui->hot = input.hot;
    for (size_t i = 0; i < CUI_COUNTOF(fpas_month_navigation); ++i)
        if (input.masked[i]) ui->kpressed[fpas_month_navigation[i]] = input.key[i];
}

/** Draws CUI's calendar and maps navigation, selection and activation back to the model. */
int fpas_cui_month_calendar(cui_ctx *ui, const char *id, int *month, int *selected,
    const fpas_month_event *events, int count, int first_weekday, float cell_height,
    const char *const *months, const char *const *days, int today, int flags, int focus, int *parts, int *activated) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const cui_id grid = key ^ 0x6A1Du;
    const cui_id selection_key = key ^ 0x6A21u;
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
        native[i] = (cui_month_event){ e->title, fpas_month_date(e->start),
            fpas_month_date(e->finish), e->time, e->all_day != 0, e->tone };
    }
    const cui_date local_today = fpas_month_date(today < 0 ? fpas_date_number(cui_date_today()) : today);
    const cui_date displayed = *month < 0 ? local_today : fpas_month_date(*month);
    cui_date shown = displayed, day = fpas_month_date(*selected);
    int *previous_selection = cui_state(ui, selection_key, sizeof *previous_selection);
    cui__mc_state *state = fpas_month_focus(ui, key, shown, day, local_today, previous_selection);
    const cui_month_opts options = { .selected = &day, .first_weekday = first_weekday,
        .cell_height = cell_height, .month_names = months, .day_names = days,
        .hour12 = (flags & 2) != 0, .today = local_today };
    const fpas_month_input input = fpas_month_limit_input(ui, displayed, state, grid, previous, next, first_weekday);
    cui_push_disabled(ui, !enabled);
    const cui_month_result result = cui_month_calendar_ex(ui, id, &shown, native, count, &options);
    cui_pop_disabled(ui);
    fpas_month_restore_input(ui, input);

    /* Keep CUI's title node, but display the year in the model's calendar. */
    const int heading = ui->nodes[ui->nodes[node].first].first;
    const char *label = months ? months[displayed.month - 1] : cui__month_name(ui, displayed.month - 1);
    fpas_text(ui, heading, cui__frame_printf(ui, "%s %d", label, displayed.year - FPAS_MONTH_YEARS));

    shown.day = 1;
    const int proposed_month = fpas_month_number(shown);
    const int proposed_day = fpas_month_number(day);
    const bool date_valid = proposed_day >= 0;
    int changes = 0;
    if (enabled) {
        if (proposed_month >= 0) {
            *month = proposed_month;
            if (result.month_changed) changes |= 4;
        }
        if (result.day_clicked && date_valid) {
            *selected = proposed_day;
            changes |= 1;
            if (result.day_activated) changes |= 2;
        }
    }
    state->focus_day = CUI_CLAMP(state->focus_day, FPAS_MONTH_EPOCH, FPAS_MONTH_EPOCH + FPAS_MAX_DAY_NUMBER);
    *activated = enabled ? result.event : -1;
    parts[0] = fpas_find(ui, grid);
    parts[1] = fpas_find(ui, previous);
    parts[2] = fpas_find(ui, next);
    parts[3] = fpas_find(ui, current);
    fpas_own(ui, key, selection_key, grid, key ^ 0x6A1Eu);
    fpas_creative_node(ui, node, key);
    return changes;
}
