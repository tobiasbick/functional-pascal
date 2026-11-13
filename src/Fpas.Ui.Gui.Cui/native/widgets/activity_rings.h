#ifndef FPAS_CUI_ACTIVITY_RINGS_H
#define FPAS_CUI_ACTIVITY_RINGS_H
#include "more_charts.h"
/* Native rendering with call-scoped input and host-owned identity. */
CUI_API int fpas_cui_activity_rings(cui_ctx *ui, const char *id, const fpas_viz_item *items, int count,
    float size, float thickness, const char *center, const char *caption, int legend, int enabled, int *node);
#endif
