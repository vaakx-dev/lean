use std::rc::Rc;

use gpui_kit::component::setting::{RenderOptions, SettingGroup, SettingItem};
use gpui_kit::{AnyElement, App, Length, SharedString, Window};

use crate::settings::{Setting, SettingValue};

pub(super) type Control = Rc<dyn Fn(&RenderOptions, &mut Window, &mut App) -> AnyElement>;

pub(crate) struct Field {
    pub(super) id: SharedString,
    pub(super) title: SharedString,
    pub(super) description: SharedString,
    pub(super) keywords: Vec<SharedString>,
    pub(super) restart: bool,
    pub(super) stacked: bool,
    pub(super) control_width: Option<Length>,
    pub(super) enabled: Option<Rc<dyn Fn(&App) -> bool>>,
    pub(super) modified: Rc<dyn Fn(&App) -> bool>,
    pub(super) reset: Rc<dyn Fn(&mut App)>,
    pub(super) control: Control,
}

impl Field {
    pub(crate) fn new(
        id: impl Into<SharedString>,
        title: impl Into<SharedString>,
        description: impl Into<SharedString>,
        modified: impl Fn(&App) -> bool + 'static,
        reset: impl Fn(&mut App) + 'static,
        control: impl Fn(&RenderOptions, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            keywords: Vec::new(),
            restart: false,
            stacked: false,
            control_width: None,
            enabled: None,
            modified: Rc::new(modified),
            reset: Rc::new(reset),
            control: Rc::new(control),
        }
    }

    pub(crate) fn bound<T: SettingValue>(
        setting: &Setting<T>,
        title: impl Into<SharedString>,
        description: impl Into<SharedString>,
        control: impl Fn(&RenderOptions, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        let modified_setting = setting.clone();
        let reset_setting = setting.clone();
        Self::new(
            setting.key,
            title,
            description,
            move |cx| modified_setting.is_modified(cx),
            move |cx| reset_setting.reset(cx),
            control,
        )
    }

    pub(crate) fn keywords<S: Into<SharedString>>(
        mut self,
        keywords: impl IntoIterator<Item = S>,
    ) -> Self {
        self.keywords.extend(keywords.into_iter().map(Into::into));
        self
    }

    pub(crate) fn requires_restart(mut self) -> Self {
        self.restart = true;
        self
    }

    pub(crate) fn stacked(mut self) -> Self {
        self.stacked = true;
        self
    }

    pub(super) fn control_width(mut self, width: impl Into<Length>) -> Self {
        self.control_width = Some(width.into());
        self
    }

    pub(crate) fn enabled_when(mut self, enabled: impl Fn(&App) -> bool + 'static) -> Self {
        self.enabled = Some(Rc::new(enabled));
        self
    }

    fn search_terms(&self) -> Vec<SharedString> {
        let mut terms = vec![
            self.title.clone(),
            self.description.clone(),
            self.id.clone(),
        ];
        terms.extend(self.keywords.iter().cloned());
        terms
    }
}

impl From<Field> for SettingItem {
    fn from(field: Field) -> Self {
        let terms = field.search_terms();
        let modified = field.modified.clone();
        let reset = field.reset.clone();
        let field = Rc::new(field);
        SettingItem::render(move |options, window, cx| field.render(options, window, cx))
            .keywords(terms)
            .on_reset(move |cx| modified(cx), move |_, cx| reset(cx))
    }
}

pub(crate) fn group(
    title: impl Into<SharedString>,
    fields: impl IntoIterator<Item = Field>,
) -> SettingGroup {
    SettingGroup::new()
        .title(title)
        .items(fields.into_iter().map(SettingItem::from))
}
