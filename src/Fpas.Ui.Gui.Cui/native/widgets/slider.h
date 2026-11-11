#ifndef FPAS_SLIDER_H
#define FPAS_SLIDER_H

/** Draws CUI's elastic slider.
 * @param ui Current UI-thread context.
 * @param id Stable slider identity.
 * @param label Borrowed literal caption.
 * @param value Supplies the current value and receives a proposed adjustment.
 * @param minimum Inclusive lower bound.
 * @param maximum Inclusive upper bound.
 * @param step Increment, or zero for continuous adjustment.
 * @param show_value Whether to show the numeric value.
 * @param decimal_places Number of decimal places.
 * @param percent Whether to append a percent sign.
 * @param enabled Whether the track accepts input.
 * @param focus Whether to request focus on the track.
 * @param track Receives the interactive track node for inspection.
 * @return Nonzero when input proposes a changed value. */
CUI_API int fpas_cui_slider(cui_ctx *ui, const char *id, const char *label, float *value,
    float minimum, float maximum, float step, int show_value, int decimal_places, int percent,
    int enabled, int focus, int *track);

#endif
