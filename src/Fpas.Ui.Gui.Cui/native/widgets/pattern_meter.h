#ifndef FPAS_CUI_PATTERN_METER_H
#define FPAS_CUI_PATTERN_METER_H
#include "charts.h"
/* Draws the native widget and preserves host identity. */
CUI_API int fpas_cui_pattern_meter(cui_ctx *ui, const char *id, const fpas_meter_part *parts, int count, const fpas_meter_options *options, int enabled, int focus, int *node, unsigned int *key);
#endif
