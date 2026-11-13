#ifndef FPAS_CUI_LINE_CHART_H
#define FPAS_CUI_LINE_CHART_H
#include "charts.h"
/* Draws the native widget and preserves host identity. */
CUI_API int fpas_cui_line_chart(cui_ctx *ui, const char *id, const fpas_chart_series *series, int count, const fpas_chart_options *options);
#endif
