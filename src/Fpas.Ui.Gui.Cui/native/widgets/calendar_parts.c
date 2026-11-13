#include "../fpas_cui.h"

/* DateOnly uses a 0001-01-01 epoch. CUI uses 1970-01-01. */
enum { FPAS_DATE_EPOCH = 719162, FPAS_MAX_DAY_NUMBER = 3652058, FPAS_CALENDAR_PARTS = 45 };

static cui_date fpas_date_from_number(int day) {
    return day < 0 ? (cui_date){0} : cui_date_from_days(day - FPAS_DATE_EPOCH);
}

static int fpas_date_number(cui_date date) {
    return date.year > 0 && date.year <= 9999 ? cui_date_to_days(date) + FPAS_DATE_EPOCH : -1;
}

static cui_id fpas_calendar_month_button(cui_ctx *ui, bool forward) {
    return cui__hash_str(forward ? CUI_ICON_CHEVRON_RIGHT : CUI_ICON_CHEVRON_LEFT,
        cui_make_id(ui, forward ? "Next month" : "Previous month"));
}

static cui_id fpas_calendar_navigation(cui_id calendar, bool forward) {
    return cui__hash_str(forward ? "#next" : "#prev", calendar);
}
