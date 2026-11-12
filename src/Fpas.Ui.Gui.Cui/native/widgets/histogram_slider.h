#ifndef FPAS_HISTOGRAM_SLIDER_H
#define FPAS_HISTOGRAM_SLIDER_H

/** Draws CUI's histogram and its range slider.
 * @param ui Current drawing context.
 * @param id Stable identity.
 * @param label Literal caption.
 * @param bins Finite nonnegative bin heights.
 * @param bin_count Number of bins.
 * @param lower Lower endpoint, updated on a proposal.
 * @param upper Upper endpoint, updated on a proposal.
 * @param minimum Inclusive lower bound.
 * @param maximum Inclusive upper bound.
 * @param height Logical chart height, or zero for CUI's default.
 * @param format Validated C numeric format.
 * @param enabled Whether the track accepts input.
 * @param focus Whether to request track focus.
 * @param track Receives the native range track node.
 * @return Whether the input proposed a change. */
CUI_API int fpas_cui_histogram_slider(cui_ctx *ui, const char *id, const char *label, const float *bins,
    int bin_count, float *lower, float *upper, float minimum, float maximum, float height,
    const char *format, int enabled, int focus, int *track);
#endif
