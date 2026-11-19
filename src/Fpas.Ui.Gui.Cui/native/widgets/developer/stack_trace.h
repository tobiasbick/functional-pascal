// [Documentation](../../../../../docs/ui/gui/more-developer-tools/stack-trace.md)
#ifndef FPAS_CUI_STACK_TRACE_H
#define FPAS_CUI_STACK_TRACE_H
/** Draws stack frames and returns an activation proposal.
 * Arrays and UTF-8 strings are borrowed for this call. Source-line arrays and
 * their strings are copied into frame storage for deferred source-preview paint.
 * @param ui Current UI-thread frame context.
 * @param id Stable widget identity.
 * @param frames Ordered frames; each source array contains source_count entries.
 * @param frames_count Number of frames, or zero for an empty trace.
 * @param title Optional trace heading.
 * @param message Optional error message.
 * @param language Optional syntax language for source previews.
 * @param width_kind FPAS extent kind.
 * @param width Width basis for that extent.
 * @param enabled Nonzero to accept input outside disabled scopes.
 * @param out Receives action as the clicked frame index, or -1. */
CUI_API void fpas_cui_stack_trace(cui_ctx *ui, const char *id, const cui_stack_frame * frames, int frames_count, const char * title, const char * message, const char * language, int width_kind, float width, int enabled, fpas_dev_result * out);
#endif
