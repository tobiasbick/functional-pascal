#ifndef FPAS_CUI_GANTT_H
#define FPAS_CUI_GANTT_H
typedef struct { const char *name, *owner; int start, finish; float progress; int tone, dependency, milestone, state; } fpas_gantt_task;
CUI_API int fpas_cui_gantt(cui_ctx *ui, const char *id, int origin, int days, fpas_gantt_task *tasks, int count,
    int today, float day_width, float row_height, int height_kind, float height, int flags, int focus, int *parts);
#endif
