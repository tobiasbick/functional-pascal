#ifndef FPAS_CUI_CHARTS_H
#define FPAS_CUI_CHARTS_H

/* Call-scoped data. CUI copies the data needed by deferred painting. */
typedef struct { const char *name; const float *values; int count, palette; uint32_t rgba; } fpas_chart_series;
typedef struct { int kind; double x, y; const char *label, *detail; int accent; } fpas_chart_annotation;
typedef struct {
    int width_kind; float width, height; int flags;
    const char *const *labels; int label_count; const char *format;
    float minimum, maximum; const fpas_chart_annotation *annotations; int annotation_count;
} fpas_chart_options;
typedef struct { const char *name; float value; int palette; uint32_t rgba; float density; } fpas_meter_part;
typedef struct {
    float total; const char *label, *unit, *format, *free_label;
    int flags; float height; int width_kind; float width;
} fpas_meter_options;

#endif
