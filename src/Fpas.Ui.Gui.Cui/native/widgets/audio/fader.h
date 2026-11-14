#ifndef FPAS_FADER_H
#define FPAS_FADER_H

/** Draws CUI's fader.
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
 * @param length Track length in logical points.
 * @param horizontal Whether the track runs left to right.
 * @param ticks Optional values to mark beside the track.
 * @param tick_count Number of tick values.
 * @param levels Optional linear channel peaks.
 * @param channels Number of channel peaks.
 * @param enabled Whether the widget accepts input.
 * @param focus Whether to request native focus.
 * @param control Receives the tracked native node.
 * @return Whether input proposed a new value. */
CUI_API int fpas_cui_fader(cui_ctx *ui, const char *id, const char *label, float *value,
    float minimum, float maximum, float step, float reset, const char *format, const char *unit,
    int hide_value, float length, int horizontal, const float *ticks, int tick_count,
    const float *levels, int channels, int enabled, int focus, int *control);
#endif
