// [Documentation](../../../../../docs/ui/gui/developer-tools/commit-graph.md)
#ifndef FPAS_CUI_COMMIT_GRAPH_H
#define FPAS_CUI_COMMIT_GRAPH_H
/** Builds the history. Selection -2 retains native state. Flags enable input (1) or request focus (2). */
CUI_API void fpas_cui_commit_graph(cui_ctx *ui, const char *id, const cui_commit *commits,
    int commits_count, int selection, float height, int width_kind, float width, int flags, fpas_dev_result *out);
#endif
