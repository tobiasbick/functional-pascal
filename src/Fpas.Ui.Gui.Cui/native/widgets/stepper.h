#ifndef FPAS_STEPPER_H
#define FPAS_STEPPER_H

/** Draws CUI's stepper inside a stable ID scope.
 * @param ui Current UI-thread context.
 * @param id Stable stepper identity.
 * @param steps Borrowed literal captions in display order.
 * @param count Positive number of steps.
 * @param current Zero-based step in progress, or count when complete. */
CUI_API void fpas_cui_stepper(cui_ctx *ui, const char *id, const char *const *steps,
    int count, int current);

#endif
