/* Keep companion-state ownership beside the CUI implementation that defines it. */
typedef struct { cui_id first, second, popup; } fpas_owned_state;

static cui_id fpas_owner_key(cui_id key) { return key ^ 0x46504153u; }

/* Positional groups share CUI's index hash and 32-item focus range. */
static void fpas_indexed_focus(cui_ctx *ui, cui_id group, int count, int focus) {
    for (int index = count; index < 32; ++index) {
        const cui_id key = cui__hash(&index, sizeof index, group);
        if (ui->focus == key) ui->focus = 0;
        if (ui->active == key) ui->active = 0;
    }
    if (focus) {
        const int index = focus - 1;
        ui->focus_next_request = cui__hash(&index, sizeof index, group);
    }
}

static void fpas_forget_state(cui_ctx *ui, cui_id key) {
    if (!key) return;
    if (ui->focus == key) ui->focus = 0;
    if (ui->active == key) ui->active = 0;
    int bucket = (int)(key & (uint32_t)(ui->bucket_count - 1));
    int *link = &ui->state_buckets[bucket];
    while (*link >= 0) {
        int index = *link;
        cui_state_entry *entry = &ui->states[index];
        if (entry->id != key) { link = &entry->next; continue; }
        *link = entry->next;
        if (entry->dtor && entry->data) entry->dtor(ui, entry->data);
        cui__free(ui, entry->data);
        memset(entry, 0, sizeof *entry);
        entry->next = ui->state_free;
        ui->state_free = index;
        return;
    }
}

static void fpas_release_owned(cui_ctx *ui, fpas_owned_state state) {
    if (state.popup) {
        cui_popup *popup = cui__popup_get(ui, state.popup, false);
        if (popup) { popup->open = false; memset(&popup->s, 0, sizeof popup->s); }
    }
    fpas_forget_state(ui, state.first);
    fpas_forget_state(ui, state.second);
}

static void fpas_own(cui_ctx *ui, cui_id key, cui_id first, cui_id second, cui_id popup) {
    fpas_owned_state *state = cui_state(ui, fpas_owner_key(key), sizeof *state);
    if (state->first != first || state->second != second || state->popup != popup)
        fpas_release_owned(ui, *state);
    *state = (fpas_owned_state){first, second, popup};
}

void fpas_cui_forget(cui_ctx *ui, unsigned int key) {
    if (!key) return;
    const cui_id owner = fpas_owner_key(key);
    const fpas_owned_state *entry = cui__state_lazy(ui, owner, sizeof *entry, false);
    if (entry) {
        fpas_release_owned(ui, *entry);
        fpas_forget_state(ui, owner);
    }
    fpas_forget_state(ui, key);
}
