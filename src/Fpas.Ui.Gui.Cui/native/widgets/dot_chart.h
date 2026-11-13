#ifndef FPAS_CUI_DOT_CHART_H
#define FPAS_CUI_DOT_CHART_H

typedef struct {
    fpas_chart_options chart;
    double x_min, x_max, x_step;
    const char *x_format, *unit;
    float pitch;
    int units, flags;
} fpas_dot_options;
CUI_API int fpas_cui_dot_chart(cui_ctx *ui, const char *id, const float *samples, const int *accent,
    int series, int count, const fpas_dot_options *o, int *index, double *x, float *value);

#endif
