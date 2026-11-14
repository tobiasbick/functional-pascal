#ifndef FPAS_CUI_MONTH_CALENDAR_H
#define FPAS_CUI_MONTH_CALENDAR_H
typedef struct { const char *title; int start, finish; float time; int all_day, tone; } fpas_month_event;
CUI_API int fpas_cui_month_calendar(cui_ctx *ui, const char *id, int *month, int *selected,
    const fpas_month_event *events, int count, int first_weekday, float cell_height,
    const char *const *months, const char *const *days, int today, int flags, int focus, int *parts, int *activated);
#endif
