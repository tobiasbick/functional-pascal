#ifndef FPAS_CUI_WAFFLE_H
#define FPAS_CUI_WAFFLE_H
#include "visualizations.h"
/* Draws the native widget with frame-owned text and host tracking. */
CUI_API int fpas_cui_waffle(cui_ctx *ui, const char *id, const fpas_viz_item *items, int count,
    float cell, int columns, int rows, const char *format, int enabled, int *node);
#endif
