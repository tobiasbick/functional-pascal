#ifndef FPAS_CUI_FUNNEL_H
#define FPAS_CUI_FUNNEL_H
#include "visualizations.h"
/* Draws the native widget with frame-owned text and host tracking. */
CUI_API int fpas_cui_funnel(cui_ctx *ui, const char *id, const fpas_viz_item *items, int count,
    int width_kind, float width, int palette, uint32_t rgba, const char *format, int enabled, int *node);
#endif
