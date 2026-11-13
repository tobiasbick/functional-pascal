#ifndef FPAS_CUI_MORE_CHARTS_H
#define FPAS_CUI_MORE_CHARTS_H
#include "visualizations.h"
typedef struct { const char *label; int palette; uint32_t rgba; } fpas_sankey_node;
typedef struct { const char *name; const cui_vec2 *points; const float *sizes; int count, palette; uint32_t rgba; } fpas_scatter_series;
typedef struct { float to; int tone; const char *name; } fpas_gauge_band;
typedef struct {
    const char *label, *unit, *format; float size, sweep; const fpas_gauge_band *bands;
    int band_count, flags; float target; int style; const char *detail; int palette; uint32_t rgba;
} fpas_radial_options;
typedef struct {
    int width_kind; float width, height; const char *const *labels; const char *format;
    const fpas_chart_series *overlays; int overlay_count, flags, up_palette; uint32_t up_rgba;
    int down_palette; uint32_t down_rgba;
} fpas_candle_options;
#endif
