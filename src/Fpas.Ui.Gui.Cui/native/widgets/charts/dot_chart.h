// [Documentation](../../../../../docs/ui/gui/instruments/dot-chart.md)
#ifndef FPAS_CUI_DOT_CHART_H
#define FPAS_CUI_DOT_CHART_H

/** Native dot-chart settings. chart supplies shared geometry, Y bounds and annotations.
 * x_min/x_max/x_step define the X axis; x_format/unit supply formatted captions.
 * pitch is logical band spacing; units selects numeric (0) or time (1) labels.
 * flags: bit 0 hides axes, bit 1 hides readout, bit 2 enables input, bit 3 requests focus.
 * Label callbacks and their user data must remain valid through frame presentation. */
typedef struct {
    fpas_chart_options chart;
    double x_min, x_max, x_step;
    const char *x_format, *unit;
    float pitch;
    int units, flags;
    cui_axis_label_fn x_label;
    void *x_label_user;
    cui_axis_label_fn value_label;
    void *value_label_user;
} fpas_dot_options;
/** Draws up to sixteen dot-chart bands.
 * @param ui Current frame context.
 * @param id Stable widget identity.
 * @param samples Call-scoped flattened band-major samples, series times count entries.
 * @param accent One accent flag per band.
 * @param series Number of bands, at most sixteen.
 * @param count Samples per band.
 * @param o Chart settings, valid for this call; callbacks remain valid through presentation.
 * @param index Receives the hovered sample index.
 * @param x Receives the X readout.
 * @param value Receives the sample-value readout.
 * @return Nonzero when enabled input activates the chart. */
CUI_API int fpas_cui_dot_chart(cui_ctx *ui, const char *id, const float *samples, const int *accent,
    int series, int count, const fpas_dot_options *o, int *index, double *x, float *value);

#endif
