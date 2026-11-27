// [Documentation](../../../../docs/ui/gui/more-charts/README.md)
/* Pixel parity and pointer contracts against direct pinned CUI calls. */
#include "../fpas_cui.h"
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct { cui_ctx *ui; cui_sw_renderer *sw; unsigned char *pixels; int selected; } sample;
static int failures, comparisons;
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"line %d: %s\n",__LINE__,#x); ++failures; } } while(0)
static sample create(int width, bool dark) {
    sample s={0}; s.selected=-1; s.sw=cui_sw_create();
    s.ui=cui_create(&(cui_ctx_desc){.renderer=cui_sw_iface(s.sw)});
    cui_set_viewport(s.ui,width,600,1);cui_set_reduced_motion(s.ui,true);
    cui_set_theme(s.ui,dark?CUI_THEME_DARK:CUI_THEME_LIGHT);cui_input_mouse_leave(s.ui);
    s.pixels=malloc(700*600*4);return s;
}
static void destroy(sample s) { free(s.pixels);cui_destroy(s.ui);cui_sw_destroy(s.sw); }

static int waffle_gutter_frame(cui_ctx *ui) {
    const fpas_viz_item items[] = { { "One", 4, 0, 0 }, { "Two", 2, 0, 0 } };
    int node;
    cui_begin_frame(ui, .016f);
    const int clicked = fpas_cui_waffle(ui, "gutter", items, 2, 14, 2, 2, NULL, 1, &node);
    cui_end_frame(ui);
    return clicked;
}

static bool waffle_gutter_contract(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 480, 420, 1);
    cui_set_reduced_motion(ui, true);
    waffle_gutter_frame(ui);
    const cui_rect r = cui_get_rect(ui, cui_make_id(ui, "gutter"));
    const struct { float x, y; int result; } clicks[] = { {7, 7, 0}, {15, 7, -1}, {7, 15, -1}, {60, 14, 0} };
    bool valid = true;
    for (int test = 0; test < (int)(sizeof clicks / sizeof clicks[0]); ++test) {
        cui_input_mouse_move(ui, r.x + clicks[test].x, r.y + clicks[test].y);
        waffle_gutter_frame(ui);
        cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        waffle_gutter_frame(ui);
        cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
        valid &= waffle_gutter_frame(ui) == clicks[test].result;
    }
    cui_destroy(ui);
    if (!valid) fprintf(stderr, "Waffle activated a gutter or ignored a square or legend entry.\n");
    return valid;
}

static int calendar_skip(sample *s) {
    int base=cui_date_to_days((cui_date){2026,1,5}), wd=((base+3)%7+7)%7;
    int weeks=(wd+364+6)/7;
    cui_rect r=cui_get_rect(s->ui,cui_make_id(s->ui,"widget"));
    int fit=(int)floorf((r.w-32+3)/14);if(fit<1)fit=1;
    return fit<weeks?(weeks-fit)*7-wd:0;
}

static int draw(sample *s, bool bridge, int kind, int width, bool enabled, bool extras) {
    float matrix[12]={1,2,3,4,5,6,7,8,9,10,11,12}, days[364];
    for(int i=0;i<364;i++) days[i]=(float)(i%13);
    float a[]={82,64,90,58,75,88},b[]={70,85,72,80,60,79};
    const char *rows[]={"Mon","Tue","Wed"},*columns[]={"0h",NULL,"6h","9h"};
    const char *axes[]={"Speed","Size","Polish","Docs","Platforms","Tests"};
    cui_series series[]={{"1.0",a,6,{0}},{"1.1",b,6,CUI_COLOR_SERIES(3)}};
    fpas_chart_series bs[]={{"1.0",a,6,0,0},{"1.1",b,6,4,0}};
    cui_viz_item items[]={{"Layout",38,{0}},{"Text",24,{0}},{"Paint",21,{0}},{"Input",9,{0}},{"Other",8,{0}}};
    fpas_viz_item bi[]={{"Layout",38,0,0},{"Text",24,0,0},{"Paint",21,0,0},{"Input",9,0,0},{"Other",8,0,0}};
    if(extras) { items[1].color=(cui_color){17,109,90,255}; bi[1].palette=-1;bi[1].rgba=0xff5a6d11u; }
    int result=-1,node;
    cui_begin_frame(s->ui,.04f);
    cui_column(s->ui,.pad=16,.gap=12) {
        if(!bridge)cui_push_disabled(s->ui,!enabled);
        switch(kind) {
        case 0:
            if(bridge)result=fpas_cui_heatmap(s->ui,"widget",matrix,3,4,rows,columns,extras?18:0,0,extras?20:0,extras?2:0,0,"%.0f req",extras?3:0,extras?80:0,enabled,&node);
            else result=cui_heatmap(s->ui,"widget",matrix,3,4,.row_labels=rows,.col_labels=columns,.cell=extras?18:0,.min=0,.max=extras?20:0,.format="%.0f req",
                .color=extras?CUI_COLOR_SERIES(1):(cui_color){0},.width=extras?CUI_PCT(80):(cui_size){0});
            break;
        case 1: {
            if(bridge)result=fpas_cui_calendar_heatmap(s->ui,"widget",2026,1,5,days,364,extras?9:0,extras?2:0,0,"%.0f","contributions",enabled,&node);
            else {
                result=cui_calendar_heatmap(s->ui,"widget",((cui_date){2026,1,5}),days,364,.cell=extras?9:0,
                    .color=extras?CUI_COLOR_SERIES(1):(cui_color){0},.format="%.0f",.unit="contributions");
                if(result>=0)result+=calendar_skip(s);
            }
            break;
        }
        case 2:
            if(bridge)result=fpas_cui_treemap(s->ui,"widget",bi,5,extras?3:1,extras?80:0,240,"%.0f KB",1,&s->selected,enabled,&node);
            else result=cui_treemap(s->ui,"widget",items,5,.width=extras?CUI_PCT(80):CUI_FILL,.height=240,.format="%.0f KB",.selected=&s->selected);
            break;
        case 3:
            if(bridge)result=fpas_cui_radar_chart(s->ui,"widget",bs,2,axes,6,220,100,"%.0f",!extras,enabled,&node);
            else result=cui_radar_chart(s->ui,"widget",series,2,axes,6,.size=220,.max=100,.format="%.0f",.no_legend=extras);
            break;
        case 4:
            if(bridge)result=fpas_cui_funnel(s->ui,"widget",bi,5,extras?2:1,extras?240:0,extras?2:0,0,"%.0f",enabled,&node);
            else result=cui_funnel(s->ui,"widget",items,5,.width=extras?CUI_PX(240):CUI_FILL,.color=extras?CUI_COLOR_SERIES(1):(cui_color){0},.format="%.0f");
            break;
        case 5:
            if(bridge)result=fpas_cui_split_bar(s->ui,"widget",bi,5,extras?3:1,extras?75:0,12,"%.0f ms",!extras,enabled,&node);
            else result=cui_split_bar(s->ui,"widget",items,5,.width=extras?CUI_PCT(75):CUI_FILL,.height=12,.format="%.0f ms",.no_legend=extras);
            break;
        case 6:
            if(bridge)result=fpas_cui_waffle(s->ui,"widget",bi,5,extras?9:14,extras?3:10,extras?2:10,"%.0f",enabled,&node);
            else result=cui_waffle(s->ui,"widget",items,5,.cell=extras?9:14,.columns=extras?3:10,.rows=extras?2:10,.format="%.0f");
            break;
        }
        if(!bridge)cui_pop_disabled(s->ui);
    }
    memset(matrix,0,sizeof matrix);memset(days,0,sizeof days);memset(a,0,sizeof a);memset(b,0,sizeof b);
    cui_end_frame(s->ui);
    cui_sw_render(s->sw,cui_get_draw_data(s->ui),s->pixels,width,600,width*4,CUI_SW_RGBA);
    return result;
}

static int draw_grid(sample *s, bool bridge, bool calendar) {
    const float days[] = { 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14 };
    const cui_date start = { 2026, 1, 5 };
    int clicked, node;
    cui_begin_frame(s->ui, .04f);
    if (calendar) {
        clicked = bridge ? fpas_cui_calendar_heatmap(s->ui, "grid", start.year, start.month, start.day,
            days, 14, 0, 0, 0, NULL, NULL, 1, &node)
            : cui_calendar_heatmap_ex(s->ui, "grid", start, days, 14, NULL);
    } else {
        clicked = bridge ? fpas_cui_heatmap(s->ui, "grid", days, 2, 2,
            NULL, NULL, 18, 0, 0, 0, 0, NULL, 0, 0, 1, &node)
            : cui_heatmap(s->ui, "grid", days, 2, 2, .cell = 18);
    }
    cui_end_frame(s->ui);
    cui_sw_render(s->sw, cui_get_draw_data(s->ui), s->pixels, 320, 600, 320 * 4, CUI_SW_RGBA);
    return clicked;
}

static void heatmap_gaps(bool calendar) {
    sample raw = create(320, false), bridge = create(320, false);
    draw_grid(&raw, false, calendar);
    draw_grid(&bridge, true, calendar);
    const size_t bytes = 320 * 600 * 4;
    unsigned char *unhovered = malloc(bytes);
    memcpy(unhovered, bridge.pixels, bytes);
    const cui_rect r = cui_get_rect(bridge.ui, cui_make_id(bridge.ui, "grid"));
    const float x = r.x + (calendar ? 32 : 0), y = r.y + (calendar ? 20 : 0);
    const float cell = calendar ? 11 : 18;
    for (int direction = 0; direction < 2; ++direction) {
        for (int mark = 0; mark < 2; ++mark) {
            const float offset = cell + (mark ? 4 : 1);
            const float px = x + (direction ? 4 : offset), py = y + (direction ? offset : 4);
            cui_input_mouse_move(raw.ui, px, py);
            cui_input_mouse_move(bridge.ui, px, py);
            cui_input_mouse_button(raw.ui, CUI_MOUSE_LEFT, true);
            cui_input_mouse_button(bridge.ui, CUI_MOUSE_LEFT, true);
            CHECK(draw_grid(&raw, false, calendar) == -1);
            CHECK(draw_grid(&bridge, true, calendar) == -1);
            cui_input_mouse_button(raw.ui, CUI_MOUSE_LEFT, false);
            cui_input_mouse_button(bridge.ui, CUI_MOUSE_LEFT, false);
            const int next = direction ? (calendar ? 1 : 2) : (calendar ? 7 : 1);
            CHECK(draw_grid(&raw, false, calendar) == (mark ? next : 0));
            CHECK(draw_grid(&bridge, true, calendar) == (mark ? next : -1));
            CHECK((memcmp(bridge.pixels, unhovered, bytes) != 0) == (mark != 0));
        }
    }
    free(unhovered);
    destroy(raw);
    destroy(bridge);
}

static void draw_radar_legend(sample *s, bool bridge) {
    const char *axes[] = { "X", "Y", "Z" };
    const float values[] = { 1, 2, 3 };
    const cui_series series[] = {
        { "Version 1", values, 3, { 0 } }, { "Version 2", values, 3, { 0 } },
        { "Version 3", values, 3, { 0 } }, { "Version 4", values, 3, { 230, 195, 23, 255 } }
    };
    const fpas_chart_series adapter[] = {
        { "Version 1", values, 3, 0, 0 }, { "Version 2", values, 3, 0, 0 },
        { "Version 3", values, 3, 0, 0 }, { "Version 4", values, 3, -1, 0xff17c3e6u }
    };
    int node;
    cui_begin_frame(s->ui, .04f);
    if (bridge) fpas_cui_radar_chart(s->ui, "radar", adapter, 4, axes, 3, 160, 4, NULL, 1, 1, &node);
    else cui_radar_chart(s->ui, "radar", series, 4, axes, 3, .size = 160, .max = 4);
    cui_button(s->ui, "Following content");
    cui_end_frame(s->ui);
}

static float last_legend_swatch(cui_ctx *ui, cui_rect chart) {
    const cui_draw_data *draw = cui_get_draw_data(ui);
    float bottom = 0;
    for (int i = 0; i < draw->vertex_count; ++i) {
        const cui_vertex *v = &draw->vertices[i];
        if (v->mode == CUI_SHADE_RRECT && v->y > chart.y + 160 &&
            v->color.r == 230 && v->color.g == 195 && v->color.b == 23 && v->y > bottom)
            bottom = v->y;
    }
    CHECK(bottom > 0);
    return bottom;
}

static void radar_legend_wrap(void) {
    sample raw = create(700, false), bridge = create(700, false);
    const int widths[] = { 700, 230, 700 };
    for (int i = 0; i < 3; ++i) {
        const bool narrow = widths[i] < 300;
        cui_set_viewport(raw.ui, widths[i], 600, 1);
        cui_set_viewport(bridge.ui, widths[i], 600, 1);
        draw_radar_legend(&raw, false);
        draw_radar_legend(&bridge, true);
        if (i > 0) CHECK(cui_needs_redraw(bridge.ui));
        for (int frame = 0; frame < 3; ++frame) {
            draw_radar_legend(&raw, false);
            draw_radar_legend(&bridge, true);
        }
        CHECK(!cui_needs_redraw(bridge.ui));
        const cui_rect a = cui_get_rect(raw.ui, cui_make_id(raw.ui, "radar"));
        const cui_rect b = cui_get_rect(bridge.ui, cui_make_id(bridge.ui, "radar"));
        const float raw_swatch = last_legend_swatch(raw.ui, a);
        const float bridge_swatch = last_legend_swatch(bridge.ui, b);
        const cui_rect sibling = cui_get_rect(bridge.ui, cui_make_id(bridge.ui, "Following content"));
        CHECK(a.w == b.w && (narrow ? b.h > a.h : b.h == a.h));
        CHECK((raw_swatch > a.y + a.h) == narrow);
        CHECK(raw_swatch == bridge_swatch && bridge_swatch <= b.y + b.h);
        CHECK(sibling.y >= b.y + b.h);
    }
    destroy(raw);
    destroy(bridge);
}

int main(void) {
    heatmap_gaps(false);
    heatmap_gaps(true);
    radar_legend_wrap();
    waffle_gutter_contract();
    for(int dark=0;dark<2;dark++)for(int compact=0;compact<2;compact++)for(int kind=0;kind<7;kind++) {
        int initial=compact?300:700;
        sample a=create(initial,dark),b=create(initial,dark);
        for(int frame=0;frame<16;frame++) {
            int width=frame>=8&&frame<12?(compact?700:300):initial;
            cui_set_viewport(a.ui,width,600,1);cui_set_viewport(b.ui,width,600,1);
            if(frame==3||frame==4||frame==5||frame==6||frame==7) {
                cui_rect r=cui_get_rect(a.ui,cui_make_id(a.ui,"widget"));
                float x=r.x+10,y=r.y+10;
                if(kind==0){x+=50;}
                if(kind==1){x+=30;y+=20;}
                if(kind==3){x=r.x+r.w/2;y=r.y+40;}
                cui_input_mouse_move(a.ui,x,y);cui_input_mouse_move(b.ui,x,y);
            } else {cui_input_mouse_leave(a.ui);cui_input_mouse_leave(b.ui);}
            if(frame==4||frame==5||frame==6||frame==7) {
                bool down=frame==4||frame==6;
                cui_input_mouse_button(a.ui,CUI_MOUSE_LEFT,down);
                cui_input_mouse_button(b.ui,CUI_MOUSE_LEFT,down);
            }
            int ar=draw(&a,false,kind,width,frame!=7,frame>=12),br=draw(&b,true,kind,width,frame!=7,frame>=12);
            CHECK(ar==br);CHECK(a.selected==b.selected);
            cui_rect ra=cui_get_rect(a.ui,cui_make_id(a.ui,"widget")), rb=cui_get_rect(b.ui,cui_make_id(b.ui,"widget"));
            CHECK(ra.x==rb.x && ra.y==rb.y && ra.w==rb.w && ra.h==rb.h);
            if(frame==5)CHECK(ar>=0);
            if(memcmp(a.pixels,b.pixels,(size_t)width*600*4)) {
                fprintf(stderr,"parity kind=%d width=%d dark=%d frame=%d\n",kind,width,dark,frame);failures++;
            }
            comparisons++;
        }
        destroy(a);destroy(b);
    }
    printf("%d strict visualization pixel comparisons, %d failures\n",comparisons,failures);
    return failures?1:0;
}
