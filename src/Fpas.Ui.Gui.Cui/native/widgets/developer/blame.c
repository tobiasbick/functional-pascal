// [Documentation](../../../../../docs/ui/gui/more-developer-tools/blame.md)
#include "../../fpas_cui.h"

/** Measures the text painted outside the blame gutter for content sizing. */
static float fpas_blame_content_width(cui_ctx *ui, const char *const *lines, int count, int first_line) {
    const float size = cui__fs(ui, 12.5f);
    float source_width = 0;
    for (int i = 0; lines && i < count; ++i) {
        if (!lines[i]) continue;
        const int length = (int)CUI_MIN(strlen(lines[i]), (size_t)2000);
        const float line_width = cui__text_width(ui, CUI_FONT_MONO, size, lines[i], length, true);
        source_width = CUI_MAX(source_width, line_width);
    }
    const char *last = cui__frame_printf(ui, "%d", CUI_MAX(1, first_line) + CUI_MAX(0, count - 1));
    const float code_width = source_width + cui__text_width(ui, CUI_FONT_MONO, size, last, -1, true) + 24;
    const float gutter = CUI_CLAMP(code_width * (0.32f / 0.68f), 200.0f, 300.0f);
    return CUI_MAX(CUI__BLM_NARROW, gutter + code_width);
}

/** Draws source and reports the revision hovered or clicked in the gutter. */
void fpas_cui_blame(cui_ctx *ui, const char *id, const char *const *lines, int lines_count,
    const int *line_commits, int line_commits_count, const cui_blame_commit *revisions, int revisions_count,
    float height, const char *language, int64_t now, int first_line, int width_kind, float width,
    int enabled, fpas_dev_result *out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const bool allowed = enabled && !cui_is_disabled(ui);
    cui_push_id(ui, id);
    const cui_id viewport = cui_make_id(ui, "lines");
    cui_pop_id(ui);
    // CUI-136 (docs/future/cui-upstream-bugs.md#cui-136): the virtual list handles wheel input in disabled scopes.
    fpas_document_scroll_input(ui, viewport, allowed);
    *out = fpas_dev_empty();
    if (!lines || lines_count < 0) lines_count = 0;
    if (line_commits_count == 0) line_commits = NULL;
    else if (line_commits_count != lines_count) lines_count = 0;
    /* The caller's buffers can reuse addresses between frames. */
    const cui_id state_key = key ^ 0xB1A3u;
    fpas_own(ui, key, state_key, viewport, 0);
    cui__blm_state *cache = cui__state_lazy(ui, state_key, sizeof *cache, false);
    const int previous_hover = cache ? cache->hovered : -1;
    if (cache) cache->lines = NULL;
    const cui_blame_opts o = {
        .width = fpas_extent(width_kind, width), .height = height,
        .lang = language ? cui__frame_strdup(ui, language, -1) : NULL, .now = now, .first_line = first_line };
    cui_push_disabled(ui, !allowed);
    const cui_blame_result r = cui_blame_ex(ui, id, lines, line_commits, lines_count, revisions, revisions_count, &o);
    cui_pop_disabled(ui);
    const int scroll = fpas_find(ui, viewport);
    if (!allowed && scroll >= 0) ui->nodes[scroll].paint = fpas_disabled_document_scroll_layout;
    if (r.hovered != previous_hover) cui_request_redraw(ui);
    if (ui->node_count == root) cui__leaf(ui, key, CUI_FIT, CUI_FIT);
    if (width_kind == 0) {
        ui->nodes[root].w = CUI_FIT;
        ui->nodes[root].self_align = CUI__ALIGN_NOSTRETCH;
        ui->nodes[root].pref_hint = fpas_blame_content_width(ui, lines, lines_count, first_line);
    }
    fpas_dev_watch_width(ui, root, key);
    fpas_creative_node(ui, root, key);
    out->action = r.clicked;
    out->hover = r.hovered;
}
