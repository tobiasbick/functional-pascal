// [Documentation](../../../../docs/ui/gui/more-charts/README.md)
/* Native chart pixels, geometry, input and retained-state retirement. */
#include "../fpas_cui.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>

typedef struct { cui_ctx *ui; cui_sw_renderer *sw; unsigned char *pixels; } sample;
static int failures, comparisons;
#define CHECK(x) do { if(!(x)){fprintf(stderr,"line %d: %s\n",__LINE__,#x);failures++;} } while(0)
static sample create(int width, bool dark) {
    sample s={0};s.sw=cui_sw_create();s.ui=cui_create(&(cui_ctx_desc){.renderer=cui_sw_iface(s.sw)});
    cui_set_viewport(s.ui,width,440,1);cui_set_reduced_motion(s.ui,true);
    cui_set_theme(s.ui,dark?CUI_THEME_DARK:CUI_THEME_LIGHT);cui_input_mouse_leave(s.ui);
    s.pixels=malloc(720*440*4);return s;
}
static void destroy(sample s){free(s.pixels);cui_destroy(s.ui);cui_sw_destroy(s.sw);}

static int draw(sample *s,bool bridge,int kind,int width,int frame) {
    const char *names[]={"Search","Trial","Paid","Bounced"};
    fpas_sankey_node bn[]={{"Search"},{"Trial"},{"Paid"},{"Bounced"}};
    cui_sankey_link links[]={{0,1,12},{0,3,8},{1,2,12}};
    cui_viz_item items[]={{"A",frame>=8?12:38,{0}},{"B",24,{0}},{"C",frame>=8?42:21,{0}}};
    fpas_viz_item bi[]={{"A",items[0].value},{"B",24},{"C",items[2].value}};
    cui_viz_item rings[]={{"Move",.82f,{0}},{"Exercise",1.24f,{0}},{"Stand",.58f,{0}}};
    fpas_viz_item br[]={{"Move",.82f},{"Exercise",1.24f},{"Stand",.58f}};
    cui_vec2 points[]={{1,2},{2,3},{3,1},{4,4}};
    float sizes[]={10,20,30,40};
    cui_scatter_series scatter[] = {
        {"Series", points, sizes, 4, {0}}, {"Second", points + 1, NULL, 2, CUI_COLOR_WARNING}
    };
    fpas_scatter_series bs[] = {
        {"Series", points, sizes, 4, 0, 0}, {"Second", points + 1, NULL, 2, -CUI_TOKEN_WARNING - 1, 0}
    };
    cui_gauge_band bands[]={{60,CUI_TONE_SUCCESS,"Normal"},{85,CUI_TONE_WARNING,"Busy"},{100,CUI_TONE_DANGER,"Overloaded"}};
    fpas_gauge_band bb[]={{60,CUI_TONE_SUCCESS,"Normal"},{85,CUI_TONE_WARNING,"Busy"},{100,CUI_TONE_DANGER,"Overloaded"}};
    cui_candle candles[120];const char *labels[120];float overlay[120];
    for(int i=0;i<120;i++){float p=64+i*.2f;candles[i]=(cui_candle){p,p+1,p-1,p+.5f,2000+i*10};labels[i]=i%2?"Period":NULL;overlay[i]=p+.3f;}
    cui_series ov={"Average",overlay,120,{0}}; fpas_chart_series bov={"Average",overlay,120,0,0};
    int node,result=-1;bool enabled=frame!=7,extra=frame>=12;
    cui_begin_frame(s->ui,.04f);
    cui_column(s->ui,.pad=16,.gap=12) {
        if(!bridge)cui_push_disabled(s->ui,!enabled);
        switch(kind) {
        case 0: {
            cui_color colors[] = { CUI_TOKEN_COLOR(CUI_TOKEN_SERIES_2), {0}, {0}, CUI_RGB(0x4d78aa) };
            if (extra) {
                bn[0].palette = 2;
                bn[3].palette = -1;
                bn[3].rgba = 0xffaa784du;
            }
            if(bridge)result=fpas_cui_sankey(s->ui,"widget",bn,4,links,3,extra?2:1,280,240,extra?9.5f:0,"%.0f",enabled,&node);
            else result=cui_sankey(s->ui,"widget",names,4,links,3,.width=extra?CUI_PX(280):CUI_FILL,
                .height=240,.node_width=extra?9.5f:0,.node_colors=extra?colors:NULL,.format="%.0f");
            break;
        }
        case 1: {
            items[1].color = CUI_RGB(0x287878);
            bi[1].palette = -1;
            bi[1].rgba = 0xff787828u;
            if(bridge)result=fpas_cui_bar_list(s->ui,"widget",bi,3,extra?2:1,250,"%.0f",extra?2:0,extra,extra?3:0,0,enabled,&node);
            else result=cui_bar_list(s->ui,"widget",items,3,.width=extra?CUI_PX(250):CUI_FILL,.format="%.0f",
                .max_rows=extra?2:0,.keep_order=extra,.color=extra?CUI_TOKEN_COLOR(CUI_TOKEN_SERIES_3):(cui_color){0});
            break;
        }
        case 2: {
            const char *center = frame == 12 ? NULL : frame == 13 ? "" : "82%";
            const char *caption = frame == 14 ? NULL : frame == 15 ? "" : "of goal";
            float thickness = extra ? 8.5f : 0;
            if(bridge)result=fpas_cui_activity_rings(s->ui,"widget",br,3,170,thickness,center,caption,extra,enabled,&node);
            else result=cui_activity_rings(s->ui,"widget",rings,3,.size=170,.thickness=thickness,
                .center=center,.caption=caption,.legend=extra);
            break;
        }
        case 3:
            if (extra) { scatter[0].sizes = NULL; bs[0].sizes = NULL; }
            if (bridge) result = fpas_cui_scatter_chart(s->ui, "widget", bs, 2, 1, 0, 260,
                extra ? "" : "Nodes", extra ? NULL : "Frame time", "%.0f", "%.2f", !extra, enabled, &node);
            else result = cui_scatter_chart(s->ui, "widget", scatter, 2, .height = 260,
                .x_label = extra ? "" : "Nodes", .y_label = extra ? NULL : "Frame time",
                .x_format = "%.0f", .y_format = "%.2f", .no_legend = extra);
            break;
        case 4:case 6:case 7: {
            int style=kind==4?CUI_GAUGE_SEGMENTS:kind==6?CUI_GAUGE_ARC:CUI_GAUGE_RING;
            fpas_radial_options o={.label="CPU",.unit="%",.size=180,.bands=bb,.band_count=3,.flags=extra?2:3,
                .target=85,.style=style,.detail=extra?"of goal":NULL,
                .sweep=extra?210:0,.format=extra?"%.1f":NULL,.palette=extra?-CUI_TOKEN_INFO-1:0};
            if(bridge)fpas_cui_radial_gauge(s->ui,"widget",64,0,100,&o);
            else cui_radial_gauge(s->ui,"widget",64,0,100,.label="CPU",.unit="%",.size=180,.bands=bands,.band_count=3,
                .show_target=true,.target=85,.style=style,.detail=extra?"of goal":NULL,.ticks=extra?-1:0,
                .sweep=extra?210:0,.format=extra?"%.1f":NULL,.color=extra?CUI_COLOR_INFO:(cui_color){0});
            break;
        }
        case 5: {
            fpas_candle_options o={.width_kind=1,.height=280,.labels=extra?labels:NULL,.overlays=extra?&bov:NULL,
                .overlay_count=extra?1:0,.flags=extra?3:0,.format=extra?"%.1f":NULL};
            if(bridge)result=fpas_cui_candlestick(s->ui,"widget",candles,frame>=16?120:100,&o,enabled,0,&node);
            else result=cui_candlestick(s->ui,"widget",candles,frame>=16?120:100,.height=280,.labels=extra?labels:NULL,
                .overlays=extra?&ov:NULL,.overlay_count=extra?1:0,.ohlc=extra,.no_volume=extra,
                .format=extra?"%.1f":NULL);
            break;
        }
        }
        if(!bridge)cui_pop_disabled(s->ui);
    }
    memset(points,0,sizeof points);memset(sizes,0,sizeof sizes);memset(candles,0,sizeof candles);memset(overlay,0,sizeof overlay);
    cui_end_frame(s->ui);
    cui_sw_render(s->sw,cui_get_draw_data(s->ui),s->pixels,width,440,width*4,CUI_SW_RGBA);return result;
}

/* CUI allocations include persistent item states; all replacement labels have equal length. */
typedef struct {size_t bytes;} memory;
typedef union {size_t bytes;long double align;} allocation;
static void *tracked_alloc(size_t bytes,void *user){allocation *p=malloc(sizeof *p+bytes);if(!p)abort();p->bytes=bytes;((memory*)user)->bytes+=bytes;return p+1;}
static void tracked_free(void *ptr,void *user){if(!ptr)return;allocation *p=(allocation*)ptr-1;((memory*)user)->bytes-=p->bytes;free(p);}
static void *tracked_realloc(void *ptr,size_t bytes,void *user){
    if(!ptr)return tracked_alloc(bytes,user);if(!bytes){tracked_free(ptr,user);return NULL;}
    allocation *old=(allocation*)ptr-1;size_t prior=old->bytes;
    allocation *p=realloc(old,sizeof *p+bytes);if(!p)abort();p->bytes=bytes;((memory*)user)->bytes+=bytes-prior;return p+1;
}
/** Steps the guarded FPAS viewport with its current dataset and input. */
static int candle_cursor(cui_ctx *ui, const cui_candle *candles, int count, cui_key key, bool focus) {
    if (key != CUI_KEY_NONE) {
        cui_input_key(ui, key, true, 0);
        cui_input_key(ui, key, false, 0);
    }
    int node;
    const fpas_candle_options options = { .width_kind = 1, .height = 280 };
    cui_begin_frame(ui, .04f);
    const int cursor = fpas_cui_candlestick(ui, "price", candles, count, &options, 1, focus, &node);
    cui_end_frame(ui);
    return cursor;
}

/** Checks only FPAS for bounded indices while replacing shorter and empty data. */
static void candle_replacement(void) {
    sample s = create(720, false);
    cui_candle data[120];
    for (int i = 0; i < 120; ++i) {
        const float open = 64 + i * .2f;
        data[i] = (cui_candle){open, open + 1, open - 1, open + .5f, 2000};
    }
    CHECK(candle_cursor(s.ui, data, 120, CUI_KEY_NONE, true) == -1);
    CHECK(candle_cursor(s.ui, data, 120, CUI_KEY_END, false) == 119);
    CHECK(candle_cursor(s.ui, data, 5, CUI_KEY_NONE, false) == 4);
    CHECK(candle_cursor(s.ui, NULL, 0, CUI_KEY_NONE, false) == -1);
    CHECK(candle_cursor(s.ui, data, 12, CUI_KEY_NONE, false) == -1);
    CHECK(candle_cursor(s.ui, data, 12, CUI_KEY_HOME, false) == 0);
    CHECK(candle_cursor(s.ui, data, 12, CUI_KEY_END, false) == 11);
    destroy(s);
}

/** Verifies keyboard focus adds a ring without replacing the chart painter. */
static void candle_focus_paint(void) {
    enum { WIDTH = 720, HEIGHT = 440 };
    sample base = create(WIDTH, false), focused = create(WIDTH, false);
    for (int frame = 0; frame < 3; ++frame) {
        draw(&base, true, 5, WIDTH, frame);
        draw(&focused, true, 5, WIDTH, frame);
    }
    const cui_id chart = cui_make_id(focused.ui, "widget");
    cui_input_key(focused.ui, CUI_KEY_TAB, true, 0);
    cui_input_key(focused.ui, CUI_KEY_TAB, false, 0);
    draw(&base, true, 5, WIDTH, 3);
    draw(&focused, true, 5, WIDTH, 3);
    const cui_rect bounds = cui_get_rect(focused.ui, chart);
    CHECK(cui_focus_visible(focused.ui, chart));
    CHECK(memcmp(base.pixels, focused.pixels, WIDTH * HEIGHT * 4) != 0);
    bool interior_matches = true;
    const int x = (int)bounds.x + 8;
    const size_t bytes = (size_t)((int)bounds.w - 16) * 4;
    for (int y = (int)bounds.y + 8; y < (int)(bounds.y + bounds.h) - 8; ++y) {
        const size_t offset = ((size_t)y * WIDTH + x) * 4;
        interior_matches &= memcmp(base.pixels + offset, focused.pixels + offset, bytes) == 0;
    }
    CHECK(interior_matches);

    cui_input_mouse_move(focused.ui, bounds.x + 10, bounds.y + 60);
    cui_input_mouse_button(focused.ui, CUI_MOUSE_LEFT, true);
    draw(&focused, true, 5, WIDTH, 4);
    CHECK(cui_focused_id(focused.ui) == chart && !cui_focus_visible(focused.ui, chart));
    cui_input_mouse_button(focused.ui, CUI_MOUSE_LEFT, false);
    draw(&focused, true, 5, WIDTH, 5);
    cui_input_key(focused.ui, CUI_KEY_HOME, true, 0);
    cui_input_key(focused.ui, CUI_KEY_HOME, false, 0);
    CHECK(draw(&focused, true, 5, WIDTH, 6) == 0);
    CHECK(cui_focus_visible(focused.ui, chart));

    cui_popover_open(focused.ui, "first");
    cui_popover_open(focused.ui, "second");
    cui_input_key(focused.ui, CUI_KEY_END, true, 0);
    cui_input_key(focused.ui, CUI_KEY_END, false, 0);
    CHECK(draw(&focused, true, 5, WIDTH, 8) == 99);
    CHECK(cui_popover_is_open(focused.ui, "first"));
    CHECK(cui_popover_is_open(focused.ui, "second"));
    destroy(base);
    destroy(focused);
}

/** Leaves the enclosing clickable card in charge of the passive gauge face. */
static void gauge_passes_card_click(void) {
    sample s = create(720, false);
    bool clicked = false;
    const fpas_radial_options options = { .size = 180, .label = "CPU" };
    for (int frame = 0; frame < 4; ++frame) {
        if (frame == 2) {
            const cui_rect gauge = cui_get_rect(s.ui, cui_make_id(s.ui, "gauge"));
            cui_input_mouse_move(s.ui, gauge.x + gauge.w / 2, gauge.y + gauge.h / 2);
            cui_input_mouse_button(s.ui, CUI_MOUSE_LEFT, true);
        }
        if (frame == 3) cui_input_mouse_button(s.ui, CUI_MOUSE_LEFT, false);
        cui_begin_frame(s.ui, .04f);
        clicked = cui_card_begin(s.ui, &(cui_card_opts){ .id = "card", .width = CUI_PX(260),
            .height = CUI_PX(260), .hoverable = true });
        fpas_cui_radial_gauge(s.ui, "gauge", 42, 0, 100, &options);
        cui_card_end(s.ui);
        cui_end_frame(s.ui);
    }
    CHECK(clicked);
    destroy(s);
}

static void retirement(void) {
    memory m={0};cui_sw_renderer *sw=cui_sw_create();
    cui_ctx *ui=cui_create(&(cui_ctx_desc){.renderer=cui_sw_iface(sw),.allocator={tracked_alloc,tracked_realloc,tracked_free,&m}});
    cui_set_viewport(ui,640,440,1);cui_set_reduced_motion(ui,true);size_t baseline=0;
    for(int round=0;round<240;round++){
        char label[32];snprintf(label,sizeof label,"item-%03d",round);
        fpas_viz_item item={label,42};int node;
        cui_begin_frame(ui,.04f);
        fpas_cui_bar_list(ui,"list",&item,1,1,0,NULL,0,0,0,0,1,&node);
        cui_end_frame(ui);
        if(round==119)baseline=m.bytes;
    }
    CHECK(m.bytes==baseline);
    fpas_cui_forget(ui,cui_make_id(ui,"list"));
    CHECK(m.bytes<baseline);
    cui_candle candle={64,65,63,64.5f,2000};
    fpas_candle_options options={.width_kind=1,.height=120};
    for(int round=0;round<240;round++){
        char id[32];snprintf(id,sizeof id,"price-%03d",round);int node;
        cui_begin_frame(ui,.04f);
        fpas_cui_candlestick(ui,id,&candle,1,&options,1,0,&node);
        cui_end_frame(ui);fpas_cui_forget(ui,cui_make_id(ui,id));
        if(round==119)baseline=m.bytes;
    }
    CHECK(m.bytes==baseline);
    cui_destroy(ui);cui_sw_destroy(sw);CHECK(m.bytes==0);
}
/* A queued wheel event must leave a disabled viewport unchanged. */
static int disabled_wheel(bool bridge) {
    sample a=create(720,false),b=create(720,false);
    for(int frame=0;frame<7;frame++){
        cui_input_mouse_move(a.ui,100,100);cui_input_mouse_move(b.ui,100,100);
        draw(&a,bridge,5,720,frame);draw(&b,bridge,5,720,frame);
    }
    cui_input_scroll(b.ui,0,3);draw(&a,bridge,5,720,7);draw(&b,bridge,5,720,7);
    int changed=memcmp(a.pixels,b.pixels,720*440*4)!=0;destroy(a);destroy(b);return changed;
}

/* A ribbon click reports the original link index, including skipped inputs. */
static int sankey_click(bool bridge) {
    sample s=create(720,false);int result=-1;
    /* Direct CUI borrows these arrays until cui_end_frame paints the chart. */
    const char *names[]={"Source","Paid"};cui_sankey_link links[]={{0,1,0},{0,1,42}};
    fpas_sankey_node nodes[]={{"Source"},{"Paid"}};int node;
    for(int frame=0;frame<6;frame++){
        if(frame>=3)cui_input_mouse_move(s.ui,300,120);
        if(frame==4||frame==5)cui_input_mouse_button(s.ui,CUI_MOUSE_LEFT,frame==4);
        cui_begin_frame(s.ui,.04f);
        cui_column(s.ui,.pad=16){
            if(bridge)result=fpas_cui_sankey(s.ui,"flow",nodes,2,links,2,1,0,220,0,NULL,1,&node);
            else result=cui_sankey(s.ui,"flow",names,2,links,2,.height=220);
        }
        cui_end_frame(s.ui);
    }
    destroy(s);return result;
}

static void bar_list_animation(void) {
    sample s = create(720, false);
    cui_set_reduced_motion(s.ui, false);
    int clicked = -1, node;
    for (int frame = 0; frame <= 52; ++frame) {
        if (frame >= 50) cui_input_mouse_move(s.ui, 34, 33);
        if (frame == 51 || frame == 52) cui_input_mouse_button(s.ui, CUI_MOUSE_LEFT, frame == 51);
        fpas_viz_item items[] = {{"First",frame >= 51 ? 50 : 100,-1,0xff0000ffu},
            {"Second",frame >= 51 ? 100 : 50,-1,0xffff0000u}};
        cui_begin_frame(s.ui, 1.0f / 60);
        cui_column(s.ui, .pad=16) {
            clicked = fpas_cui_bar_list(s.ui, "list", items, 2, 1, 0, NULL, 0, 0, 0, 0, 1, &node);
        }
        cui_end_frame(s.ui);
    }
    cui_sw_render(s.sw, cui_get_draw_data(s.ui), s.pixels, 720, 440, 720 * 4, CUI_SW_RGBA);
    const unsigned char *pixel = s.pixels + (33 * 720 + 200) * 4;
    CHECK(pixel[0] > pixel[2]);
    CHECK(clicked == 0);
    destroy(s);
}

static int rings_click(bool legend, bool legend_row) {
    sample s = create(720, false);
    const fpas_viz_item rings[] = {{"A",.8f},{"B",.8f},{"C",.8f},{"D",.8f},{"E",.8f}};
    int clicked = -1, node;
    for (int frame = 0; frame < 5; ++frame) {
        if (frame >= 2) cui_input_mouse_move(s.ui, legend_row ? 200 : 91, legend_row ? 143 : 91);
        if (frame == 3 || frame == 4) cui_input_mouse_button(s.ui, CUI_MOUSE_LEFT, frame == 3);
        cui_begin_frame(s.ui, .04f);
        cui_column(s.ui, .pad=16) {
            clicked = fpas_cui_activity_rings(s.ui, "rings", rings, 5, 0, 0, NULL, NULL, legend, 1, &node);
        }
        cui_end_frame(s.ui);
    }
    destroy(s);
    return clicked;
}

int main(void) {
    for(int dark=0;dark<2;dark++)for(int narrow=0;narrow<2;narrow++)for(int kind=0;kind<8;kind++){
        int initial=narrow?320:720;sample a=create(initial,dark),b=create(initial,dark);
        for(int frame=0;frame<20;frame++){
            int width=frame>=8&&frame<12?(narrow?720:320):initial;
            cui_set_viewport(a.ui,width,440,1);cui_set_viewport(b.ui,width,440,1);
            float x=100,y=65;
            if(kind==2){x=100;y=22;}
            if(frame>=3&&frame<=7){cui_input_mouse_move(a.ui,x,y);cui_input_mouse_move(b.ui,x,y);}
            else {cui_input_mouse_leave(a.ui);cui_input_mouse_leave(b.ui);}
            if(frame==4||frame==5){bool down=frame==4;cui_input_mouse_button(a.ui,CUI_MOUSE_LEFT,down);cui_input_mouse_button(b.ui,CUI_MOUSE_LEFT,down);}
            if(kind==5 && frame==6){cui_input_scroll(a.ui,0,2);cui_input_scroll(b.ui,0,2);}
            if(kind==5 && frame>=16 && frame<=18){
                cui_input_mouse_move(a.ui,frame==16?100:160,120);cui_input_mouse_move(b.ui,frame==16?100:160,120);
                cui_input_mouse_button(a.ui,CUI_MOUSE_LEFT,frame!=18);cui_input_mouse_button(b.ui,CUI_MOUSE_LEFT,frame!=18);
            }
            int ar=draw(&a,false,kind,width,frame),br=draw(&b,true,kind,width,frame);
            CHECK(ar==br);
            cui_rect ra=cui_get_rect(a.ui,cui_make_id(a.ui,"widget")),rb=cui_get_rect(b.ui,cui_make_id(b.ui,"widget"));
            CHECK(ra.x==rb.x&&ra.y==rb.y&&ra.w==rb.w&&ra.h==rb.h);
            if(memcmp(a.pixels,b.pixels,(size_t)width*440*4)){
                fprintf(stderr,"parity kind=%d frame=%d width=%d dark=%d\n",kind,frame,width,dark);failures++;
            }
            comparisons++;
        }
        destroy(a);destroy(b);
    }
    retirement();
    candle_replacement();
    candle_focus_paint();
    gauge_passes_card_click();
    CHECK(disabled_wheel(false)==0);CHECK(disabled_wheel(true)==0);
    CHECK(sankey_click(false)==3);CHECK(sankey_click(true)==3);
    bar_list_animation();
    CHECK(rings_click(false, false) == -1);
    CHECK(rings_click(true, false) == -1);
    CHECK(rings_click(true, true) == 4);
    printf("%d strict more-chart pixel/geometry comparisons, %d failures\n",comparisons,failures);
    return failures?1:0;
}
