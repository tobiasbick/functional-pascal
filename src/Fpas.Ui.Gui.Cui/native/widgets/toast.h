#ifndef FPAS_TOAST_H
#define FPAS_TOAST_H

/** Queues a native toast for CUI's end-of-frame overlay.
 * @param ui Current UI-thread context.
 * @param title Borrowed literal title.
 * @param description Borrowed secondary text, or NULL.
 * @param tone Native toast tone.
 * @param duration Seconds, zero for the native default, negative to keep.
 * @param action Borrowed button caption, or NULL.
 * @param timer Whether to show the timer line.
 * @return The native toast handle. */
CUI_API int fpas_cui_toast(cui_ctx *ui, const char *title, const char *description,
    int tone, float duration, const char *action, int timer);

/** Reads finalized toast nodes and consumes its action event, if any.
 * @param ui Current UI-thread context after presentation.
 * @param handle Native toast handle.
 * @param card Receives the visible card node, or -1.
 * @param action Receives the visible action node, or -1.
 * @param dismiss Receives the visible close node, or -1.
 * @return Bit 0 if active, bit 1 if the action was invoked. */
CUI_API int fpas_cui_toast_inspect(cui_ctx *ui, int handle, int *card, int *action, int *dismiss);

#endif
