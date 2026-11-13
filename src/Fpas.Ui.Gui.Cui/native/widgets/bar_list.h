#ifndef FPAS_CUI_BAR_LIST_H
#define FPAS_CUI_BAR_LIST_H
#include "more_charts.h"
/* Native rendering with call-scoped input and host-owned identity. */
CUI_API int fpas_cui_bar_list(cui_ctx *ui, const char *id, const fpas_viz_item *items, int count,
    int width_kind, float width, const char *format, int maximum_rows, int keep_order,
    int palette, uint32_t rgba, int enabled, int *node);
#endif
