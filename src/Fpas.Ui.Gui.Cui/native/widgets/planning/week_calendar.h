#ifndef FPAS_CUI_WEEK_CALENDAR_H
#define FPAS_CUI_WEEK_CALENDAR_H
typedef struct { const char *title, *detail; int day; float start, finish; int tone; } fpas_week_event;
CUI_API int fpas_cui_week_calendar(cui_ctx *ui, const char *id, int week, fpas_week_event *events, int count,
    float start, float finish, float hour_height, int height_kind, float height, int flags, int focus, int *parts);
#endif
