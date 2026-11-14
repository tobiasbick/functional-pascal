#ifndef FPAS_CUI_HEATMAP_H
#define FPAS_CUI_HEATMAP_H
#include "visualizations.h"
/* Draws the native widget with frame-owned text and host tracking. */
CUI_API int fpas_cui_heatmap(cui_ctx *ui, const char *id, const float *values, int rows, int columns,
    const char *const *row_labels, const char *const *column_labels, float cell, float minimum, float maximum,
    int palette, uint32_t rgba, const char *format, int width_kind, float width, int enabled, int *node);
#endif
