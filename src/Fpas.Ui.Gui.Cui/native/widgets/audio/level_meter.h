#ifndef FPAS_LEVEL_METER_H
#define FPAS_LEVEL_METER_H

/** Draws CUI's level meter.
 * @param ui Current drawing context.
 * @param id Stable widget identity.
 * @param levels Linear channel peaks.
 * @param channels Number of channel peaks.
 * @param minimum_db Lower bound in dBFS.
 * @param maximum_db Upper bound in dBFS.
 * @param length Bar length in logical points.
 * @param thickness Thickness of each channel bar.
 * @param horizontal Whether the bars run left to right.
 * @param scale Whether to label the dB scale.
 * @param segmented Whether to draw LED segments.
 * @param hold Peak hold in seconds.
 * @param decay Fall rate in dB per second.
 * @param enabled Whether the widget accepts input.
 * @param control Receives the tracked native node.
 * @return Whether the clip light was clicked. */
CUI_API int fpas_cui_level_meter(cui_ctx *ui, const char *id, const float *levels,
    int channels, float minimum_db, float maximum_db, float length, float thickness,
    int horizontal, int scale, int segmented, float hold, float decay, int enabled, int *control);
#endif
