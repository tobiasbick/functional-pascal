#ifndef FPAS_CUI_DONUT_CHART_H
#define FPAS_CUI_DONUT_CHART_H
#include "charts.h"
/* Draws the native widget and preserves host identity. */
CUI_API int fpas_cui_donut_chart(cui_ctx *ui, const char *id, const float *values, int count, const fpas_chart_options *options);
#endif
