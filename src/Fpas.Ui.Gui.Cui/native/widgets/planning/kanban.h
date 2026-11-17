#ifndef FPAS_CUI_KANBAN_H
#define FPAS_CUI_KANBAN_H
/** Begins a board and its disabled scope.
 * @param ui Current frame context.
 * @param id Stable board identity.
 * @param enabled Nonzero to accept input outside disabled scopes.
 * @return Root node to pass to fpas_cui_kanban_end. */
CUI_API int fpas_cui_kanban_begin(cui_ctx *ui, const char *id, int enabled);
/** Begins a column in the current board.
 * @param ui Current frame context.
 * @param title Column caption.
 * @param count Model card count for the heading.
 * @param palette Automatic (0), theme series (1..6), or explicit RGBA (-1).
 * @param color Explicit color with red in the low byte.
 * @param limit Work-in-progress limit, or zero to hide it.
 * @param width Logical column width, or zero for the native default.
 * @param parts Receives column and heading node indices.
 * @return Nonzero when enabled input activates the column's add action. */
CUI_API int fpas_cui_kanban_column_begin(cui_ctx *ui, const char *title, int count, int palette, uint32_t color, int limit, float width, int *parts);
/** Begins a card body in the current column.
 * @param ui Current frame context.
 * @param key Stable nonnegative card key, unique across the board.
 * @param focus Nonzero to request card focus.
 * @param node Receives the card root node.
 * @return Nonzero when enabled input activates the card. */
CUI_API int fpas_cui_kanban_card_begin(cui_ctx *ui, int key, int focus, int *node);
/** Ends the current card body.
 * @param ui Current frame context. */
CUI_API void fpas_cui_kanban_card_end(cui_ctx *ui);
/** Ends the current column.
 * @param ui Current frame context. */
CUI_API void fpas_cui_kanban_column_end(cui_ctx *ui);
/** Ends the board and balances its disabled scope.
 * @param ui Current frame context.
 * @param root Root returned by fpas_cui_kanban_begin.
 * @param move Receives the proposed move; apply removal before insertion.
 * @return Nonzero when enabled input proposes a move. */
CUI_API int fpas_cui_kanban_end(cui_ctx *ui, int root, cui_kanban_move *move);
#endif
