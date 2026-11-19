// [Documentation](../../../../../docs/ui/gui/code/log-console.md)
#ifndef FPAS_CUI_LOG_CONSOLE_H
#define FPAS_CUI_LOG_CONSOLE_H
typedef struct { const char *text; double time; int level; } fpas_log_entry;
/** Draws an application-owned snapshot of log entries.
 * The array and each NUL-terminated UTF-8 text are borrowed for this call;
 * the adapter retains its own normalized copy until forget or destruction.
 * @param ui Current UI-thread frame context.
 * @param id Stable widget identity.
 * @param entries Ordered entries, or NULL when count is zero.
 * @param count Number of entries in the snapshot.
 * @param width_kind FPAS extent kind.
 * @param width Width basis for that extent.
 * @param height Logical viewport height.
 * @param size Font size, or zero for the native default.
 * @param flags Bit 0 enables input, bit 1 shows timestamps, bit 2 hides the toolbar.
 * @return Nonzero when Clear proposes removing a nonempty application snapshot.
 * The caller must apply that proposal to prevent entries returning next frame. */
CUI_API int fpas_cui_log_console(cui_ctx *ui, const char *id, const fpas_log_entry *entries, int count, int width_kind, float width, float height, float size, int flags);
#endif
