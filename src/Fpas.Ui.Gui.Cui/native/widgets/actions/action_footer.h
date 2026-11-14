#ifndef FPAS_ACTION_FOOTER_H
#define FPAS_ACTION_FOOTER_H

/** Begins CUI's native action footer row.
 * @param ui Drawing context.
 * @return Frame-local row node. */
CUI_API int fpas_cui_action_footer_begin(cui_ctx *ui);

/** Ends the footer and makes the row the current layout target.
 * @param ui Drawing context.
 * @param footer Frame-local footer row node. */
CUI_API void fpas_cui_action_footer_end(cui_ctx *ui, int footer);

#endif
