#ifndef FPAS_CUI_FLAME_GRAPH_H
#define FPAS_CUI_FLAME_GRAPH_H
typedef struct { double start, end; float scroll; int selected, zoomed; } fpas_flame_view;
CUI_API int fpas_cui_flame_graph(cui_ctx *ui, const char *id, const cui_flame_span *spans, int count, fpas_flame_view *view, double minimum, float height, float row_height, int width_kind, float width, int flags);
#endif
