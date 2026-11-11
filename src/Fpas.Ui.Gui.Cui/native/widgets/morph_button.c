/* Literal text adaptation shared by CUI's morphing button integrations. */
typedef struct {
    cui_spring width;
    bool initialized;
} fpas_morph_width;

static fpas_morph_width fpas_morph_begin(cui_ctx *ui, cui__morph_state *state,
    const char *slot, const char *icon, cui_control_size size) {
    const fpas_morph_width saved = { state->w, state->init };
    const cui__morph_face face = { .icon = icon, .label = slot };
    /* A settled slot width must not request endless frames for literal captions. */
    state->w.x = cui__face_width(ui, &face, size);
    state->w.v = 0;
    return saved;
}

static void fpas_morph_end(cui_ctx *ui, cui__morph_state *state, fpas_morph_width saved,
    const char *caption, const char *glyph, cui_control_size size) {
    float target = cui__btn_pad(size) * 2;
    if (*caption) target += cui__text_width(ui, CUI_FONT_MEDIUM, cui__btn_font(ui, size),
                                          caption, (int)strlen(caption), false);
    if (glyph) target += cui__btn_icon(size) + (*caption ? 6 : 0);
    state->w = saved.width;
    if (!saved.initialized) { state->w.x = target; state->w.v = 0; }
    cui__spring(ui, &state->w, target, CUI_SPRING_MORPH);
    ui->nodes[ui->last_node].w = CUI_PX(CUI_MAX(cui__control_height(ui, size), state->w.x));
}

static void fpas_morph_captions(cui_ctx *ui, const char *slots, const char *const *captions) {
    for (int i = ui->last_node + 1; i < ui->node_count; ++i) {
        cui_node *text = &ui->nodes[i];
        if (!(text->flags & CUI_NF_TEXT) || text->font == CUI_FONT_ICONS) continue;
        const char *slot = strchr(slots, text->text[0]);
        if (!slot) continue;
        const char *literal = captions[slot - slots];
        text->text = cui__frame_strdup(ui, literal, (int)strlen(literal));
        text->text_len = (int)strlen(literal);
    }
}
