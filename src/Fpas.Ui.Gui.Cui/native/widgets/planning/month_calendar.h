// [Documentation](../../../../../docs/ui/gui/planning/month-calendar.md)
#ifndef FPAS_CUI_MONTH_CALENDAR_H
#define FPAS_CUI_MONTH_CALENDAR_H
/** Call-scoped appointment: title, inclusive start/finish .NET day numbers,
 * fractional-hour time, all-day flag and native tone. */
typedef struct { const char *title; int start, finish; float time; int all_day, tone; } fpas_month_event;
/** Draws a month calendar and returns controlled navigation and selection proposals.
 * @param ui Current frame context.
 * @param id Stable widget identity.
 * @param month Displayed .NET day number; receives the proposed month's first day.
 * @param selected Selected .NET day number or minus one; receives the proposal.
 * @param events Call-scoped appointment array.
 * @param count Number of appointments.
 * @param first_weekday First weekday, Monday zero.
 * @param cell_height Logical cell height, or zero for native sizing.
 * @param months Optional twelve localized month captions.
 * @param days Optional seven weekday captions starting with Monday.
 * @param today Current .NET day number, or minus one for the native current date.
 * @param flags Bit 0 enables input; bit 1 selects twelve-hour formatting.
 * @param focus Zero requests nothing; 1 grid, 2 previous, 3 next, 4 Today.
 * @param parts Receives four node indices in focus order, or minus one when absent.
 * @param activated Receives the activated input appointment index, or minus one.
 * @return Bit 0 day clicked, bit 1 day activated, bit 2 month changed; zero when disabled. */
CUI_API int fpas_cui_month_calendar(cui_ctx *ui, const char *id, int *month, int *selected,
    const fpas_month_event *events, int count, int first_weekday, float cell_height,
    const char *const *months, const char *const *days, int today, int flags, int focus, int *parts, int *activated);
#endif
