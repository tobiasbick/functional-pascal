#ifndef FPAS_RANGE_SLIDER_H
#define FPAS_RANGE_SLIDER_H

/** Draws CUI's native two-endpoint slider.
 * @param ui Current UI-thread context.
 * @param id Stable slider identity.
 * @param label Borrowed literal caption.
 * @param lower Supplies the lower value and receives a proposed adjustment.
 * @param upper Supplies the upper value and receives a proposed adjustment.
 * @param minimum Inclusive lower bound.
 * @param maximum Inclusive upper bound.
 * @param enabled Whether the track accepts input.
 * @param focus Whether to request focus on the track.
 * @param track Receives the interactive track node for inspection.
 * @return Nonzero when input proposes changed endpoints. */
CUI_API int fpas_cui_range_slider(cui_ctx *ui, const char *id, const char *label, float *lower,
    float *upper, float minimum, float maximum, int enabled, int focus, int *track);

#endif
