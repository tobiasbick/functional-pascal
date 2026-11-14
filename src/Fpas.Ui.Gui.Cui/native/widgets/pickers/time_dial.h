#ifndef FPAS_TIME_DIAL_H
#define FPAS_TIME_DIAL_H

/** Draws CUI's time dial.
 * @param ui Current drawing context.
 * @param id Stable dial identity.
 * @param value Current time in minutes since midnight.
 * @param size Logical face diameter, or zero for CUI's default.
 * @param enabled Whether the dial accepts input.
 * @param focus Zero for no request, one for hours or two for minutes.
 * @param nodes Optional four-node array for the dial, its two selectors and the clock face.
 * @return Proposed time in minutes since midnight. */
CUI_API int fpas_cui_time_dial(cui_ctx *ui, const char *id, int value, float size,
    int enabled, int focus, int *nodes);
#endif
