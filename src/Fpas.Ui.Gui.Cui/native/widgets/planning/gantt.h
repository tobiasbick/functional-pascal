#ifndef FPAS_CUI_GANTT_H
#define FPAS_CUI_GANTT_H
/** Task whose dates are day offsets from the chart's origin. */
typedef struct { const char *name, *owner; int start, finish; float progress; int tone, dependency, milestone, state; } fpas_gantt_task;

/** Draws the native Gantt chart and writes proposed changes to task dates.
 * @param ui Current drawing context.
 * @param id Stable chart identity.
 * @param origin First date as a .NET day number.
 * @param days Number of timeline days.
 * @param tasks Current tasks with editable start and finish offsets.
 * @param count Number of tasks.
 * @param today Offset of today's marker, or zero to hide it.
 * @param day_width Logical width of a day, or zero for the default.
 * @param row_height Logical height of a row, or zero for the default.
 * @param height_kind Native height extent kind.
 * @param height Height extent value.
 * @param flags Input flags: bit 0 enables input, bit 1 disallows edits.
 * @param focus Requested task index, or minus one.
 * @param parts Receives one native node index per task, or minus one when absent.
 * @return Activated task index, or minus one. */
CUI_API int fpas_cui_gantt(cui_ctx *ui, const char *id, int origin, int days, fpas_gantt_task *tasks, int count,
    int today, float day_width, float row_height, int height_kind, float height, int flags, int focus, int *parts);
#endif
