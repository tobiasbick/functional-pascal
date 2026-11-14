#ifndef FPAS_RULER_SLIDER_H
#define FPAS_RULER_SLIDER_H

/** Draws CUI's ruler slider.
 * @param ui Current drawing context.
 * @param id Stable identity.
 * @param label Literal caption.
 * @param value Current value, updated on a proposal.
 * @param minimum Inclusive lower bound.
 * @param maximum Inclusive upper bound.
 * @param step Positive value between ticks.
 * @param major_every Number of ticks between labels.
 * @param spacing Logical points between ticks.
 * @param width Logical width, or zero to fill.
 * @param format Validated C numeric format.
 * @param unit Optional unit caption.
 * @param enabled Whether the tape accepts input.
 * @param focus Whether to request tape focus.
 * @param track Receives the native tape node.
 * @return Whether the input proposed a change. */
CUI_API int fpas_cui_ruler_slider(cui_ctx *ui, const char *id, const char *label, float *value,
    float minimum, float maximum, float step, int major_every, float spacing, float width,
    const char *format, const char *unit, int enabled, int focus, int *track);
#endif
