#ifndef FPAS_CUI_RADAR_CHART_H
#define FPAS_CUI_RADAR_CHART_H
#include "visualizations.h"
/* Draws the native widget with frame-owned text and host tracking. */
CUI_API int fpas_cui_radar_chart(cui_ctx *ui, const char *id, const fpas_chart_series *series, int count,
    const char *const *axes, int axis_count, float size, float maximum, const char *format, int legend,
    int enabled, int *node);
#endif
