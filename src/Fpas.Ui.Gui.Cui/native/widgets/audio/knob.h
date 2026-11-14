#ifndef FPAS_KNOB_H
#define FPAS_KNOB_H

/** Draws CUI's knob.
 * @param ui Current drawing context.
 * @param id Stable widget identity.
 * @param label Literal control caption.
 * @param value Current model value, replaced on an input proposal.
 * @param minimum Inclusive lower bound.
 * @param maximum Inclusive upper bound.
 * @param step Value increment, or zero for continuous motion.
 * @param reset Value to restore on reset.
 * @param format Validated numeric C format.
 * @param unit Optional literal unit caption.
 * @param hide_value Whether to suppress the numeric value.
 * @param size Diameter in logical points.
 * @param bipolar Whether the arc starts halfway through the sweep.
 * @param log_scale Whether rotation changes the value logarithmically.
 * @param rgba Packed RGBA arc color, or zero for the default.
 * @param enabled Whether the widget accepts input.
 * @param focus Whether to request native focus.
 * @param control Receives the tracked native node.
 * @return Whether input proposed a new value. */
CUI_API int fpas_cui_knob(cui_ctx *ui, const char *id, const char *label, float *value,
    float minimum, float maximum, float step, float reset, const char *format, const char *unit,
    int hide_value, float size, int bipolar, int log_scale, uint32_t rgba, int enabled, int focus, int *control);
#endif
