#ifndef FPAS_CUI_TERMINAL_H
#define FPAS_CUI_TERMINAL_H
/** Draws cumulative ANSI output and proposes terminal input.
 * output and prompt are borrowed NUL-terminated UTF-8 strings for this call.
 * Replacing or shrinking output rebuilds scrollback; changing revision also
 * resets it. The adapter owns the retained output and releases it on forget.
 * @param ui Current UI-thread frame context.
 * @param id Stable widget identity.
 * @param output Complete cumulative output, not just the appended suffix.
 * @param revision Application revision used to reset scrollback.
 * @param prompt Input-line caption.
 * @param width_kind FPAS extent kind.
 * @param width Width basis for that extent.
 * @param height Logical height, or the native auto/fill sentinel.
 * @param size Font size, or zero for the native default.
 * @param flags Bit 0 enables input, bit 1 shows the input line, bit 2 requests focus.
 * @param interrupt Receives nonzero when enabled input proposes Ctrl+C.
 * @return Submitted command, or NULL. Copy the borrowed command before the next
 * frame or context destruction. */
CUI_API const char *fpas_cui_terminal(cui_ctx *ui, const char *id, const char *output, int revision, const char *prompt, int width_kind, float width, float height, float size, int flags, int *interrupt);
#endif
