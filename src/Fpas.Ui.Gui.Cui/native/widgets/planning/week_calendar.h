#ifndef FPAS_CUI_WEEK_CALENDAR_H
#define FPAS_CUI_WEEK_CALENDAR_H
/** Appointment placed by its day offset and the hours when it starts and finishes. */
typedef struct { const char *title, *detail; int day; float start, finish; int tone; } fpas_week_event;

/** Draws the native week calendar and writes proposed appointment changes.
 * @param ui Current drawing context.
 * @param id Stable calendar identity.
 * @param week First date of the week as a .NET day number.
 * @param events Current appointments with editable day and hour offsets.
 * @param count Number of appointments.
 * @param start First visible hour.
 * @param finish Last visible hour.
 * @param hour_height Logical height of an hour, or zero for the default.
 * @param height_kind Native height extent kind.
 * @param height Height extent value.
 * @param flags Input flags: bit 0 enables input, bit 1 disallows edits.
 * @param focus Requested appointment index, or minus one.
 * @param parts Receives one native node index per appointment, or minus one when absent.
 * @return Activated appointment index, or minus one. */
CUI_API int fpas_cui_week_calendar(cui_ctx *ui, const char *id, int week, fpas_week_event *events, int count,
    float start, float finish, float hour_height, int height_kind, float height, int flags, int focus, int *parts);
#endif
