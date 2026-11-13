#ifndef FPAS_CUI_SCATTER_CHART_H
#define FPAS_CUI_SCATTER_CHART_H
#include "more_charts.h"
/* Native rendering with call-scoped input and host-owned identity. */
CUI_API int fpas_cui_scatter_chart(cui_ctx *ui, const char *id, const fpas_scatter_series *series, int count,
    int width_kind, float width, float height, const char *x_label, const char *y_label, const char *x_format,
    const char *y_format, int legend, int enabled, int *node);
#endif
