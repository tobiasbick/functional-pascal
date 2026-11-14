#ifndef FPAS_CUI_QUERY_BUILDER_H
#define FPAS_CUI_QUERY_BUILDER_H
/** Draws the query builder and edits the supplied rules when editing is enabled.
 * @param ui Current UI context.
 * @param id Stable query builder identity.
 * @param fields Available query fields.
 * @param field_count Number of entries in fields.
 * @param rules Writable rule buffer with room for capacity entries.
 * @param count Number of populated rules, updated on edits.
 * @param capacity Number of slots in rules.
 * @param max_depth Maximum nesting depth, or zero for the native default.
 * @param read_only Nonzero to render without modifying rules.
 * @param width_kind FPAS width extent kind.
 * @param width Width basis for the chosen extent.
 * @param enabled Nonzero to allow input outside disabled scopes.
 * @param out Receives the change flag. */
CUI_API void fpas_cui_query_builder(cui_ctx *ui, const char *id, const cui_query_field *fields,
    int field_count, cui_query_node *rules, int *count, int capacity, int max_depth,
    int read_only, int width_kind, float width, int enabled, fpas_dev_result *out);
#endif
