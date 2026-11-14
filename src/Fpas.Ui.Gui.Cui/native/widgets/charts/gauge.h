#ifndef FPAS_CUI_GAUGE_H
#define FPAS_CUI_GAUGE_H
#include "charts.h"
/* Draws the native widget and preserves host identity. */
CUI_API int fpas_cui_gauge(cui_ctx *ui, const char *id, const char *label, float value, float size);
#endif
