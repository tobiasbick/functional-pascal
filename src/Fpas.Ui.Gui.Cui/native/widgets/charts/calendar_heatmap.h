#ifndef FPAS_CUI_CALENDAR_HEATMAP_H
#define FPAS_CUI_CALENDAR_HEATMAP_H
#include "visualizations.h"
/* Draws the native widget with frame-owned text and host tracking. */
CUI_API int fpas_cui_calendar_heatmap(cui_ctx *ui, const char *id, int year, int month, int day,
    const float *values, int count, float cell, int palette, uint32_t rgba, const char *format,
    const char *unit, int enabled, int *node);
#endif
