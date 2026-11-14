#ifndef FPAS_COUNTDOWN_H
#define FPAS_COUNTDOWN_H

/** Draws CUI's countdown.
 * @param ui Current drawing context.
 * @param id Stable widget identity.
 * @param seconds_left Current remaining seconds.
 * @param compact Whether to draw compact digits.
 * @param hide_days Whether to fold days into hours.
 * @return The native countdown root */
CUI_API int fpas_cui_countdown(cui_ctx *ui, const char *id, double seconds_left, int compact, int hide_days);
#endif
