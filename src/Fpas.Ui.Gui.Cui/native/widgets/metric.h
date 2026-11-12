#ifndef FPAS_CUI_METRIC_H
#define FPAS_CUI_METRIC_H

/* Draws a native metric with CUI-owned formatting and sparkline painting. */
CUI_API void fpas_cui_metric(cui_ctx *ui, const char *label, const char *value, float delta,
    const char *delta_label, const float *spark, int count, const char *icon,
    int width_kind, float width, int lower_is_better);

#endif
