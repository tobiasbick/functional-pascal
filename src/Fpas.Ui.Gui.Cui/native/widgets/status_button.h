#ifndef FPAS_STATUS_BUTTON_H
#define FPAS_STATUS_BUTTON_H

/** Draws native model-controlled work feedback with literal captions.
 * @param ui Current UI-thread context.
 * @param id Stable native identity independent of the captions.
 * @param label Idle caption.
 * @param status Idle, busy, success or error ordinal.
 * @param busy_label Busy caption.
 * @param success_label Success caption.
 * @param error_label Error caption.
 * @param icon Optional idle glyph, or NULL.
 * @param variant Shared resting button variant ordinal.
 * @param size Native control size ordinal.
 * @param enabled Whether the control accepts focus and idle input.
 * @param focus Whether to request keyboard focus.
 * @return Nonzero only on activation while idle. */
CUI_API int fpas_cui_status_button(cui_ctx *ui, const char *id, const char *label, int status,
    const char *busy_label, const char *success_label, const char *error_label, const char *icon,
    int variant, int size, int enabled, int focus);

#endif
