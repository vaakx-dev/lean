mod tab;

use gpui_kit::component::{ActiveTheme as _, Theme, h_flex};
use gpui_kit::*;

use super::EditorArea;
use tab::TabState;

const TABS_ID: &str = "editor-tabs";
const TABS_STATE: &str = "editor-tabs-state";

struct Tabs {
    area: WeakEntity<EditorArea>,
    tabs: Vec<TabState>,
    _subscriptions: [Subscription; 2],
}

impl Tabs {
    fn new(area: Entity<EditorArea>, cx: &mut Context<Self>) -> Self {
        let tabs = TabState::collect(area.read(cx));
        let subscriptions = [
            cx.observe(&area, |tabs, area, cx| tabs.sync(&area, cx)),
            cx.observe_global::<Theme>(|_, cx| cx.notify()),
        ];
        Self {
            area: area.downgrade(),
            tabs,
            _subscriptions: subscriptions,
        }
    }

    fn sync(&mut self, area: &Entity<EditorArea>, cx: &mut Context<Self>) {
        let tabs = TabState::collect(area.read(cx));
        if tabs != self.tabs {
            self.tabs = tabs;
            cx.notify();
        }
    }

    fn update_area(
        &self,
        cx: &mut App,
        update: impl FnOnce(&mut EditorArea, &mut Context<EditorArea>),
    ) {
        self.area.update(cx, update).ok();
    }
}

impl Render for Tabs {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .id(TABS_ID)
            .size_full()
            .overflow_x_scroll()
            .children(
                self.tabs
                    .iter()
                    .enumerate()
                    .map(|(index, tab)| tab.render(index, cx)),
            )
    }
}

struct TabStrip {
    area: Entity<EditorArea>,
}

impl RenderOnce for TabStrip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let area = self.area;
        window
            .use_keyed_state(TABS_STATE, cx, move |_, cx| Tabs::new(area, cx))
            .cached(StyleRefinement::default().flex_1().min_w_0().h_full())
    }
}

impl IntoElement for TabStrip {
    type Element = ViewElement<Self>;

    fn into_element(self) -> Self::Element {
        ViewElement::new(self)
    }
}

impl EditorArea {
    pub(super) fn render_tab_bar(&self, cx: &Context<Self>) -> Div {
        let theme = cx.theme();
        h_flex()
            .h(px(36.))
            .flex_shrink_0()
            .border_b_1()
            .border_color(theme.border)
            .child(TabStrip { area: cx.entity() })
            .children(self.render_file_actions(cx))
    }
}
