#ifndef FPAS_CUI_TREEMAP_H
#define FPAS_CUI_TREEMAP_H
#include "visualizations.h"
/* Draws the native widget with frame-owned text and host tracking. */
CUI_API int fpas_cui_treemap(cui_ctx *ui, const char *id, const fpas_viz_item *items, int count,
    int width_kind, float width, float height, const char *format, int selectable, int *selected, int enabled, int *node);
#endif
