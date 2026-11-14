#ifndef FPAS_ONE_TIME_CODE_H
#define FPAS_ONE_TIME_CODE_H

/** Draws native one-time code boxes and edits the supplied code.
 * @param ui Drawing context.
 * @param id Stable control identity.
 * @param code Writable ASCII buffer of length plus one bytes.
 * @param length Number of code boxes.
 * @param enabled Whether the boxes accept input.
 * @param focus Whether to request keyboard focus.
 * @param complete Receives CUI's completion event, or zero when disabled.
 * @return Native code row node. */
CUI_API int fpas_cui_one_time_code(cui_ctx *ui, const char *id, char *code, int length, int enabled, int focus, int *complete);

#endif
