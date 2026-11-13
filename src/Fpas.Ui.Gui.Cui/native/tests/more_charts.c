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
    float sizes[]={10,20,30,40};cui_scatter_series scatter={"Series",points,sizes,4,{0}};
    fpas_scatter_series bs={"Series",points,sizes,4,0,0};
    cui_gauge_band bands[]={{60,CUI_TONE_SUCCESS,"Normal"},{85,CUI_TONE_WARNING,"Busy"},{100,CUI_TONE_DANGER,"Overloaded"}};
    fpas_gauge_band bb[]={{60,CUI_TONE_SUCCESS,"Normal"},{85,CUI_TONE_WARNING,"Busy"},{100,CUI_TONE_DANGER,"Overloaded"}};
    cui_candle candles[120];const char *labels[120];float overlay[120];
    for(int i=0;i<120;i++){float p=64+i*.2f;candles[i]=(cui_candle){p,p+1,p-1,p+.5f,2000+i*10};labels[i]="Period";overlay[i]=p+.3f;}
    cui_series ov={"Average",overlay,120,{0}}; fpas_chart_series bov={"Average",overlay,120,0,0};
    int node,result=-1;bool enabled=frame!=7,extra=frame>=12;
    cui_begin_frame(s->ui,.04f);
    cui_column(s->ui,.pad=16,.gap=12) {
        if(!bridge)cui_push_disabled(s->ui,!enabled);
        switch(kind) {
        case 0:
            if(bridge)result=fpas_cui_sankey(s->ui,"widget",bn,4,links,3,1,0,240,0,"%.0f",enabled,&node);
            else result=cui_sankey(s->ui,"widget",names,4,links,3,.height=240,.format="%.0f");
            break;
        case 1:
            if(bridge)result=fpas_cui_bar_list(s->ui,"widget",bi,3,1,0,"%.0f",extra?2:0,extra,0,0,enabled,&node);
            else result=cui_bar_list(s->ui,"widget",items,3,.format="%.0f",.max_rows=extra?2:0,.keep_order=extra);
            break;
        case 2:
            if(bridge)result=fpas_cui_activity_rings(s->ui,"widget",br,3,170,0,"82%","of goal",extra,enabled,&node);
            else result=cui_activity_rings(s->ui,"widget",rings,3,.size=170,.center="82%",.caption="of goal",.legend=extra);
            break;
        case 3:
            if(extra){scatter.sizes=NULL;bs.sizes=NULL;}
            if(bridge)result=fpas_cui_scatter_chart(s->ui,"widget",&bs,1,1,0,260,"Nodes","Frame time","%.0f","%.2f",1,enabled,&node);
            else result=cui_scatter_chart(s->ui,"widget",&scatter,1,.height=260,.x_label="Nodes",.y_label="Frame time",.x_format="%.0f",.y_format="%.2f");
            break;
        case 4:case 6:case 7: {
            int style=kind==4?CUI_GAUGE_SEGMENTS:kind==6?CUI_GAUGE_ARC:CUI_GAUGE_RING;
            fpas_radial_options o={.label="CPU",.unit="%",.size=180,.bands=bb,.band_count=3,.flags=3,.target=85,.style=style,.detail=extra?"of goal":NULL};
            if(bridge)fpas_cui_radial_gauge(s->ui,"widget",64,0,100,&o);
            else cui_radial_gauge(s->ui,"widget",64,0,100,.label="CPU",.unit="%",.size=180,.bands=bands,.band_count=3,
                .show_target=true,.target=85,.style=style,.detail=extra?"of goal":NULL);
            break;
        }
        case 5: {
            fpas_candle_options o={.width_kind=1,.height=280,.labels=extra?labels:NULL,.overlays=extra?&bov:NULL,.overlay_count=extra?1:0,.flags=extra?2:0};
            if(bridge)result=fpas_cui_candlestick(s->ui,"widget",candles,frame>=16?120:100,&o,enabled,0,&node);
            else result=cui_candlestick(s->ui,"widget",candles,frame>=16?120:100,.height=280,.labels=extra?labels:NULL,
                .overlays=extra?&ov:NULL,.overlay_count=extra?1:0,.ohlc=extra);
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
/* CUI-020: the direct widget changes its disabled viewport; the adapter must not. */
static int disabled_wheel(bool bridge) {
    sample a=create(720,false),b=create(720,false);
    for(int frame=0;frame<7;frame++){
        cui_input_mouse_move(a.ui,100,100);cui_input_mouse_move(b.ui,100,100);
        draw(&a,bridge,5,720,frame);draw(&b,bridge,5,720,frame);
    }
    cui_input_scroll(b.ui,0,3);draw(&a,bridge,5,720,7);draw(&b,bridge,5,720,7);
    int changed=memcmp(a.pixels,b.pixels,720*440*4)!=0;destroy(a);destroy(b);return changed;
}

/* CUI-021: native Sankey detects the flow but returns only node clicks. */
static int sankey_click(bool bridge) {
    sample s=create(720,false);int result=-1;
    for(int frame=0;frame<6;frame++){
        if(frame>=3)cui_input_mouse_move(s.ui,300,120);
        if(frame==4||frame==5)cui_input_mouse_button(s.ui,CUI_MOUSE_LEFT,frame==4);
        cui_begin_frame(s.ui,.04f);
        cui_column(s.ui,.pad=16){
            const char *names[]={"Source","Paid"};cui_sankey_link links[]={{0,1,0},{0,1,42}};
            fpas_sankey_node nodes[]={{"Source"},{"Paid"}};int node;
            if(bridge)result=fpas_cui_sankey(s.ui,"flow",nodes,2,links,2,1,0,220,0,NULL,1,&node);
            else result=cui_sankey(s.ui,"flow",names,2,links,2,.height=220);
        }
        cui_end_frame(s.ui);
    }
    destroy(s);return result;
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
            if(kind==5&&frame>=12&&frame<16){
                cui_key key=frame==12?CUI_KEY_TAB:frame==13?CUI_KEY_HOME:frame==14?CUI_KEY_EQUAL:CUI_KEY_0;
                cui_input_key(a.ui,key,true,0);cui_input_key(a.ui,key,false,0);cui_input_key(b.ui,key,true,0);cui_input_key(b.ui,key,false,0);
            }
            int ar=draw(&a,false,kind,width,frame),br=draw(&b,true,kind,width,frame);
            /* CUI-021: the bridge also reports links on the native click frame. */
            CHECK(ar==br || (kind==0 && frame==5 && ar==-1 && br>=4));
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
    CHECK(disabled_wheel(false)==1);CHECK(disabled_wheel(true)==0);
    CHECK(sankey_click(false)==-1);CHECK(sankey_click(true)==3);
    printf("%d strict more-chart pixel/geometry comparisons, %d failures\n",comparisons,failures);
    return failures?1:0;
}
