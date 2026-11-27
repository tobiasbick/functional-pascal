// [Documentation](../../../../../docs/ui/gui/developer-tools/dependency-graph.md)
#ifndef FPAS_CUI_DEPENDENCY_GRAPH_H
#define FPAS_CUI_DEPENDENCY_GRAPH_H
/** Builds the graph. Selection -2 retains native state. Flags enable input (1) or request focus (2). */
CUI_API void fpas_cui_dependency_graph(cui_ctx *ui, const char *id, const cui_graph_node *vertices,
    int vertices_count, const cui_graph_edge *edges, int edges_count, int selection, float height, int top_down,
    int width_kind, float width, int flags, fpas_dev_result *out);
#endif
