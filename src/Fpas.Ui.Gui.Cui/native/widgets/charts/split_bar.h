#ifndef FPAS_CUI_SPLIT_BAR_H
#define FPAS_CUI_SPLIT_BAR_H
#include "visualizations.h"
/* Draws the native widget with frame-owned text and host tracking. */
CUI_API int fpas_cui_split_bar(cui_ctx *ui, const char *id, const fpas_viz_item *items, int count,
    int width_kind, float width, float height, const char *format, int legend, int enabled, int *node);
#endif
