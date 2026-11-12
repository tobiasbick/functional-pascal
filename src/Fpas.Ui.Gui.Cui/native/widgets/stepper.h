#ifndef FPAS_STEPPER_H
#define FPAS_STEPPER_H

/** Draws CUI's stepper and exposes its layout row.
 * @param ui Current UI-thread context.
 * @param steps Borrowed literal captions in display order.
 * @param count Positive number of steps.
 * @param current Zero-based step in progress, or count when complete. */
CUI_API void fpas_cui_stepper(cui_ctx *ui, const char *const *steps, int count, int current);

#endif
